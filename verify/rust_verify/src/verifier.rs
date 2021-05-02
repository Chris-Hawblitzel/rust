use crate::config::Args;
use air::ast::{CommandX, SpanOption, ValidityResult};
use rustc_interface::interface::Compiler;
use rustc_middle::ty::TyCtxt;
use rustc_span::{MultiSpan, Span};
use std::fs::File;
use std::io::Write;
use vir::ast::{Krate, VirErr, VirErrX};

pub(crate) struct Verifier {
    pub count_verified: u64,
    pub count_errors: u64,
    args: Args,
}

fn report_vir_error(compiler: &Compiler, vir_err: VirErr) {
    let span: &Span = (*vir_err.span.raw_span)
        .downcast_ref::<Span>()
        .expect("internal error: failed to cast to Span");
    let multispan = MultiSpan::from_span(*span);
    match &vir_err.x {
        VirErrX::Str(msg) => {
            compiler.session().parse_sess.span_diagnostic.span_err(multispan, &msg);
            std::process::exit(1)
        }
    }
}

fn report_verify_error(compiler: &Compiler, span1: &SpanOption, span2: &SpanOption) {
    match &**span1 {
        None => {
            panic!("internal error: found Error with no span")
        }
        Some(air::ast::Span { description, raw_span, .. }) => {
            let msg = description.as_ref().unwrap_or(&"assertion failed".to_string()).clone();
            let span: &Span =
                (*raw_span).downcast_ref::<Span>().expect("internal error: failed to cast to Span");
            let mut multispan = MultiSpan::from_span(*span);
            match &**span2 {
                None => {}
                Some(air::ast::Span { description, raw_span, .. }) => {
                    let msg =
                        description.as_ref().unwrap_or(&"related location".to_string()).clone();
                    let span: &Span = (*raw_span)
                        .downcast_ref::<Span>()
                        .expect("internal error: failed to cast to Span");
                    multispan.push_span_label(*span, msg);
                }
            }
            compiler.session().parse_sess.span_diagnostic.span_err(multispan, &msg);
        }
    }
}

fn report_chosen_triggers(
    compiler: &Compiler,
    air_span: &air::ast::Span,
    triggers: &Vec<Vec<String>>,
) {
    let span: &Span = (*air_span.raw_span)
        .downcast_ref::<Span>()
        .expect("internal error: failed to cast to Span");
    let msg = format!("chosen triggers: {:#?}", triggers);
    compiler.session().parse_sess.span_diagnostic.span_note_without_error(*span, &msg);
}

impl Verifier {
    pub fn new(args: Args) -> Verifier {
        Verifier { count_verified: 0, count_errors: 0, args: args }
    }

    fn verify(&mut self, compiler: &Compiler, krate: Krate) -> Result<(), VirErr> {
        vir::modes::check_crate(&krate)?;

        let mut z3_config = z3::Config::new();
        z3_config.set_param_value("auto_config", "false");

        let z3_context = z3::Context::new(&z3_config);
        let z3_solver = z3::Solver::new(&z3_context);
        let mut air_context = air::context::Context::new(&z3_context, &z3_solver);

        if let Some(filename) = &self.args.log_air_initial {
            let file = File::create(filename).expect(&format!("could not open file {}", filename));
            air_context.set_air_initial_log(Box::new(file));
        }
        if let Some(filename) = &self.args.log_air_final {
            let file = File::create(filename).expect(&format!("could not open file {}", filename));
            air_context.set_air_final_log(Box::new(file));
        }
        if let Some(filename) = &self.args.log_smt {
            let file = File::create(filename).expect(&format!("could not open file {}", filename));
            air_context.set_smt_log(Box::new(file));
        }

        air_context.set_z3_param("air_recommended_options", "true");
        air_context.set_rlimit(self.args.rlimit * 1000000);

        let ctx = vir::context::Ctx::new(&krate)?;

        let check_internal_result = |result| match result {
            ValidityResult::Valid => {}
            ValidityResult::TypeError(err) => {
                panic!("internal error: ill-typed AIR code: {}", err)
            }
            _ => panic!("internal error: decls should not generate queries"),
        };

        air_context.blank_line();
        air_context.comment("Prelude");
        for command in ctx.prelude().iter() {
            check_internal_result(air_context.command(&command));
        }

        air_context.blank_line();
        air_context.comment("Fuel");
        for command in ctx.fuel().iter() {
            check_internal_result(air_context.command(&command));
        }

        for function in &krate.functions {
            let commands = vir::func_to_air::func_decl_to_air(&ctx, &function)?;
            if commands.len() > 0 {
                air_context.blank_line();
                air_context.comment(&("Function-Decl ".to_string() + &function.x.name));
            }
            for command in commands.iter() {
                check_internal_result(air_context.command(&command));
            }
        }

        let commands = vir::datatype_to_air::datatypes_to_air(&krate.datatypes);
        // TODO(andrea): deduplicate
        if commands.len() > 0 {
            air_context.blank_line();
            air_context.comment(&("Datatypes".to_string()));
        }
        for command in commands.iter() {
            let result = air_context.command(&command);
            match result {
                ValidityResult::Valid => {
                    if let CommandX::CheckValid(_) = **command {
                        self.count_verified += 1;
                    }
                }
                ValidityResult::TypeError(err) => {
                    panic!("internal error: generated ill-typed AIR code: {}", err);
                }
                ValidityResult::Invalid(span_option, _) => {
                    panic!("internal error: unexpected invalid result: {:?}", span_option);
                }
            }
        }

        for function in &krate.functions {
            let commands = vir::func_to_air::func_def_to_air(&ctx, &function)?;
            if commands.len() > 0 {
                air_context.blank_line();
                air_context.comment(&("Function-Def ".to_string() + &function.x.name));
            }
            for command in commands.iter() {
                let result = air_context.command(&command);
                match result {
                    ValidityResult::Valid => {
                        if let CommandX::CheckValid(_) = **command {
                            self.count_verified += 1;
                        }
                    }
                    ValidityResult::TypeError(err) => {
                        panic!("internal error: generated ill-typed AIR code: {}", err);
                    }
                    ValidityResult::Invalid(span1, span2) => {
                        report_verify_error(compiler, &span1, &span2);
                        self.count_errors += 1;
                    }
                }
            }
        }

        if let Some(filename) = &self.args.log_triggers {
            let mut file =
                File::create(filename).expect(&format!("could not open file {}", filename));
            let chosen_triggers = ctx.get_chosen_triggers();
            for triggers in chosen_triggers {
                writeln!(file, "{:#?}", triggers)
                    .expect(&format!("error writing to file {}", filename));
            }
        }
        if self.args.show_triggers {
            let chosen_triggers = ctx.get_chosen_triggers();
            for (span, triggers) in chosen_triggers {
                report_chosen_triggers(compiler, &span, &triggers);
            }
        }

        Ok(())
    }

    fn run<'tcx>(&mut self, compiler: &Compiler, tcx: TyCtxt<'tcx>) -> Result<bool, VirErr> {
        let _ = tcx.formal_verifier_callback.replace(Some(Box::new(crate::typecheck::Typecheck {
            int_ty_id: None,
            nat_ty_id: None,
        })));
        match rustc_typeck::check_crate(tcx) {
            Ok(()) => {}
            Err(rustc_errors::ErrorReported {}) => {
                return Ok(false);
            }
        }

        let hir = tcx.hir();
        let vir_crate = crate::rust_to_vir::crate_to_vir(tcx, hir.krate())?;
        if let Some(filename) = &self.args.log_vir {
            let mut file =
                File::create(filename).expect(&format!("could not open file {}", filename));
            for datatype in vir_crate.datatypes.iter() {
                writeln!(&mut file, "datatype {} @ {:?}", datatype.x.name, datatype.span)
                    .expect("cannot write to vir file");
                writeln!(&mut file, "{:?}", datatype.x.a).expect("cannot write to vir file");
                writeln!(&mut file).expect("cannot write to vir file");
            }
            for func in vir_crate.functions.iter() {
                writeln!(&mut file, "fn {} @ {:?}", func.x.name, func.span)
                    .expect("cannot write to vir file");
                for param in func.x.params.iter() {
                    writeln!(
                        &mut file,
                        "parameter {}: {:?} @ {:?}",
                        param.x.name, param.x.typ, param.span
                    )
                    .expect("cannot write to vir file");
                }
                writeln!(&mut file, "body {:#?}", func.x.body).expect("cannot write to vir file");
                writeln!(&mut file).expect("cannot write to vir file");
            }
        }
        self.verify(&compiler, vir_crate)?;
        Ok(true)
    }
}

impl rustc_driver::Callbacks for Verifier {
    fn after_expansion<'tcx>(
        &mut self,
        compiler: &Compiler,
        queries: &'tcx rustc_interface::Queries<'tcx>,
    ) -> rustc_driver::Compilation {
        let _result = queries.global_ctxt().expect("global_ctxt").peek_mut().enter(|tcx| {
            queries.expansion().expect("expansion");
            match self.run(compiler, tcx) {
                Ok(true) => {}
                Ok(false) => {
                    std::process::exit(1);
                }
                Err(err) => {
                    report_vir_error(compiler, err);
                    std::process::exit(1);
                }
            }
        });
        rustc_driver::Compilation::Stop
    }
}
