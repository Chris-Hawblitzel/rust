use crate::context::Context;
use crate::rust_to_vir_base::{
    check_generics, def_id_to_vir_path, def_to_path_ident, get_fuel, get_mode, get_var_mode,
    get_verifier_attrs, ident_to_var, ty_to_vir, BodyCtxt,
};
use crate::rust_to_vir_expr::{expr_to_vir, pat_to_var};
use crate::util::{err_span_str, err_span_string, spanned_new, unsupported_err_span, vec_map};
use crate::{unsupported, unsupported_err, unsupported_err_unless, unsupported_unless};
use rustc_ast::Attribute;
use rustc_hir::{Body, BodyId, FnDecl, FnHeader, FnSig, Generics, Param, Unsafety};
use rustc_middle::ty::TyCtxt;
use rustc_span::symbol::Ident;
use rustc_span::Span;
use std::sync::Arc;
use vir::ast::{FunctionX, KrateX, Mode, ParamX, Typ, TypX, VirErr};
use vir::def::RETURN_VALUE;

pub(crate) fn body_to_vir<'tcx>(
    ctxt: &Context<'tcx>,
    id: &BodyId,
    body: &Body<'tcx>,
    mode: Mode,
) -> Result<vir::ast::Expr, VirErr> {
    let def = rustc_middle::ty::WithOptConstParam::unknown(id.hir_id.owner);
    let types = ctxt.tcx.typeck_opt_const_arg(def);
    let bctx = BodyCtxt { ctxt: ctxt.clone(), types, mode };
    expr_to_vir(&bctx, &body.value)
}

fn check_fn_decl<'tcx>(
    tcx: TyCtxt<'tcx>,
    decl: &'tcx FnDecl<'tcx>,
    mode: Mode,
) -> Result<Option<(Typ, Mode)>, VirErr> {
    let FnDecl { inputs: _, output, c_variadic, implicit_self } = decl;
    unsupported_unless!(!c_variadic, "c_variadic");
    match implicit_self {
        rustc_hir::ImplicitSelfKind::None => {}
        rustc_hir::ImplicitSelfKind::Imm => {}
        rustc_hir::ImplicitSelfKind::ImmRef => {}
        _ => unsupported!("implicit_self"),
    }
    match output {
        rustc_hir::FnRetTy::DefaultReturn(_) => Ok(None),
        // REVIEW: there's no attribute syntax on return types,
        // so we always return the default mode.
        // The current workaround is to return a struct if the default doesn't work.
        rustc_hir::FnRetTy::Return(ty) => Ok(Some((ty_to_vir(tcx, ty), mode))),
    }
}

pub(crate) fn check_item_fn<'tcx>(
    ctxt: &Context<'tcx>,
    vir: &mut KrateX,
    self_path: Option<vir::ast::Path>,
    id: rustc_span::def_id::DefId,
    visibility: vir::ast::Visibility,
    attrs: &[Attribute],
    sig: &'tcx FnSig<'tcx>,
    generics: &'tcx Generics,
    body_id: &BodyId,
) -> Result<(), VirErr> {
    let path = if let Some(self_path) = &self_path {
        let mut full_path = (**self_path).clone();
        Arc::make_mut(&mut full_path.segments).push(def_to_path_ident(ctxt.tcx, id));
        Arc::new(full_path)
    } else {
        def_id_to_vir_path(ctxt.tcx, id)
    };
    let mode = get_mode(Mode::Exec, attrs);
    let ret_typ_mode = match sig {
        FnSig {
            header: FnHeader { unsafety, constness: _, asyncness: _, abi: _ },
            decl,
            span: _,
        } => {
            unsupported_err_unless!(*unsafety == Unsafety::Normal, sig.span, "unsafe");
            check_fn_decl(ctxt.tcx, decl, mode)?
        }
    };
    let typ_params = check_generics(generics)?;
    let fuel = get_fuel(attrs);
    let vattrs = get_verifier_attrs(attrs)?;
    if vattrs.external {
        let mut erasure_info = ctxt.erasure_info.borrow_mut();
        erasure_info.external_functions.push(path);
        return Ok(());
    }
    let body = &ctxt.krate.bodies[body_id];
    let Body { params, value: _, generator_kind } = body;
    let mut vir_params: Vec<vir::ast::Param> = Vec::new();
    for (param, input) in params.iter().zip(sig.decl.inputs.iter()) {
        let Param { hir_id, pat, ty_span: _, span } = param;
        let name = Arc::new(pat_to_var(pat));
        fn is_self_or_self_ref(span: Span, ty: &rustc_hir::Ty) -> Result<bool, VirErr> {
            match ty.kind {
                rustc_hir::TyKind::Rptr(
                    _,
                    rustc_hir::MutTy { ty: rty, mutbl: rustc_hir::Mutability::Not, .. },
                ) => is_self_or_self_ref(span, rty),
                rustc_hir::TyKind::Path(rustc_hir::QPath::Resolved(None, path)) => match path.res {
                    rustc_hir::def::Res::SelfTy(Some(_), _impl_def_id) => {
                        unsupported_err!(span, "trait self", ty)
                    }
                    rustc_hir::def::Res::SelfTy(None, _) => Ok(true),
                    _ => Ok(false),
                },
                _ => Ok(false),
            }
        }
        let typ_args = vec_map(&typ_params, |t| Arc::new(TypX::TypParam(t.clone())));
        let typ = if is_self_or_self_ref(*span, &input)? {
            Arc::new(TypX::Datatype(
                self_path.as_ref().expect("a param is Self, so this must be an impl").clone(),
                Arc::new(typ_args),
            ))
        } else {
            ty_to_vir(ctxt.tcx, input)
        };
        let mode = get_var_mode(mode, ctxt.tcx.hir().attrs(*hir_id));
        let vir_param = spanned_new(*span, ParamX { name, typ, mode });
        vir_params.push(vir_param);
    }
    match generator_kind {
        None => {}
        _ => {
            unsupported_err!(sig.span, "generator_kind", generator_kind);
        }
    }
    let mut vir_body = body_to_vir(ctxt, body_id, body, mode)?;
    let header = vir::headers::read_header(&mut vir_body)?;
    if mode == Mode::Spec && (header.require.len() + header.ensure.len()) > 0 {
        return err_span_str(sig.span, "spec functions cannot have requires/ensures");
    }
    if header.ensure.len() > 0 {
        match (&header.ensure_id_typ, ret_typ_mode.as_ref()) {
            (None, None) => {}
            (None, Some(_)) => {
                return err_span_str(sig.span, "ensures clause must be a closure");
            }
            (Some(_), None) => {
                return err_span_str(sig.span, "ensures clause cannot be a closure");
            }
            (Some((_, typ)), Some((ret_typ, _))) => {
                if !vir::ast_util::types_equal(&typ, &ret_typ) {
                    return err_span_string(
                        sig.span,
                        format!(
                            "return type is {:?}, but ensures expects type {:?}",
                            &ret_typ, &typ
                        ),
                    );
                }
            }
        }
    }
    let params = Arc::new(vir_params);
    let (ret_name, ret_typ, ret_mode) = match (header.ensure_id_typ, ret_typ_mode) {
        (None, None) => {
            (Arc::new(RETURN_VALUE.to_string()), Arc::new(TypX::Tuple(Arc::new(vec![]))), mode)
        }
        (None, Some((typ, mode))) => (Arc::new(RETURN_VALUE.to_string()), typ, mode),
        (Some((x, _)), Some((typ, mode))) => (x, typ, mode),
        _ => panic!("internal error: ret_typ"),
    };
    let ret = spanned_new(sig.span, ParamX { name: ret_name, typ: ret_typ, mode: ret_mode });
    let func = FunctionX {
        path,
        visibility,
        mode,
        fuel,
        typ_params,
        params,
        ret,
        require: header.require,
        ensure: header.ensure,
        decrease: header.decrease,
        custom_req_err: vattrs.custom_req_err,
        hidden: Arc::new(header.hidden),
        is_abstract: vattrs.is_abstract,
        body: if vattrs.do_verify { Some(vir_body) } else { None },
    };
    let function = spanned_new(sig.span, func);
    vir.functions.push(function);
    Ok(())
}

pub(crate) fn check_foreign_item_fn<'tcx>(
    ctxt: &Context<'tcx>,
    vir: &mut KrateX,
    id: rustc_span::def_id::DefId,
    span: Span,
    visibility: vir::ast::Visibility,
    attrs: &[Attribute],
    decl: &'tcx FnDecl<'tcx>,
    idents: &[Ident],
    generics: &'tcx Generics,
) -> Result<(), VirErr> {
    let mode = get_mode(Mode::Exec, attrs);
    let ret_typ_mode = check_fn_decl(ctxt.tcx, decl, mode)?;
    let typ_params = check_generics(generics)?;
    let fuel = get_fuel(attrs);
    let mut vir_params: Vec<vir::ast::Param> = Vec::new();
    for (param, input) in idents.iter().zip(decl.inputs.iter()) {
        let name = Arc::new(ident_to_var(param));
        let typ = ty_to_vir(ctxt.tcx, input);
        // REVIEW: the parameters don't have attributes, so we use the overall mode
        let vir_param = spanned_new(param.span, ParamX { name, typ, mode });
        vir_params.push(vir_param);
    }
    let path = def_id_to_vir_path(ctxt.tcx, id);
    let params = Arc::new(vir_params);
    let (ret_typ, ret_mode) = match ret_typ_mode {
        None => (Arc::new(TypX::Tuple(Arc::new(vec![]))), mode),
        Some((typ, mode)) => (typ, mode),
    };
    let ret_param =
        ParamX { name: Arc::new(RETURN_VALUE.to_string()), typ: ret_typ, mode: ret_mode };
    let ret = spanned_new(span, ret_param);
    let func = FunctionX {
        path,
        visibility,
        fuel,
        mode,
        typ_params,
        params,
        ret,
        require: Arc::new(vec![]),
        ensure: Arc::new(vec![]),
        decrease: None,
        custom_req_err: None,
        hidden: Arc::new(vec![]),
        is_abstract: false,
        body: None,
    };
    let function = spanned_new(span, func);
    vir.functions.push(function);
    Ok(())
}
