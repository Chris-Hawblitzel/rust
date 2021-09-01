use crate::def::{prefix_box, prefix_type_id, prefix_unbox, prefix_opaque_datatype};
use crate::sst_to_air::{path_to_air_ident, typ_to_air};
use air::ast::{Command, CommandX, Commands, DeclX};
use air::ast_util::str_typ;
use std::sync::Arc;

fn datatype_to_air(datatype: &crate::ast::Datatype) -> air::ast::Datatype {
    Arc::new(air::ast::BinderX {
        name: path_to_air_ident(&datatype.x.path),
        a: Arc::new(
            datatype
                .x
                .variants
                .iter()
                .map(|variant| {
                    Arc::new(variant.map_a(|fields| {
                        Arc::new(
                            fields
                                .iter()
                                .map(|field| Arc::new(field.map_a(|(typ, _)| typ_to_air(typ))))
                                .collect::<Vec<_>>(),
                        )
                    }))
                })
                .collect::<Vec<_>>(),
        ),
    })
}

pub fn datatypes_to_air(datatypes: &crate::ast::Datatypes) -> Commands {
    let mut commands: Vec<Command> = Vec::new();
    let air_datatypes: Vec<_> =
        datatypes.iter().map(|datatype| datatype_to_air(datatype)).collect();
    commands.push(Arc::new(CommandX::Global(Arc::new(DeclX::Datatypes(Arc::new(air_datatypes))))));
    for datatype in datatypes.iter() {
        let decl_opaq_sort = Arc::new(air::ast::DeclX::Sort(
            prefix_opaque_datatype(&path_to_air_ident(&datatype.x.path))));
        commands.push(Arc::new(CommandX::Global(decl_opaq_sort)));
    }
    for opaq in &[false, true] {
        let path_to_air_ident_opaq = if !opaq {
            |path| path_to_air_ident(path)
        } else {
            |path| prefix_opaque_datatype(&path_to_air_ident(path))
        };
        for datatype in datatypes.iter() {
            let decl_type_id = Arc::new(DeclX::Const(
                prefix_type_id(&path_to_air_ident_opaq(&datatype.x.path)),
                str_typ(crate::def::TYPE),
            ));
            commands.push(Arc::new(CommandX::Global(decl_type_id)));
        }
        for datatype in datatypes.iter() {
            let decl_box = Arc::new(DeclX::Fun(
                prefix_box(&path_to_air_ident_opaq(&datatype.x.path)),
                Arc::new(vec![str_typ(&path_to_air_ident_opaq(&datatype.x.path))]),
                str_typ(crate::def::POLY),
            ));
            let decl_unbox = Arc::new(DeclX::Fun(
                prefix_unbox(&path_to_air_ident_opaq(&datatype.x.path)),
                Arc::new(vec![str_typ(crate::def::POLY)]),
                str_typ(&path_to_air_ident_opaq(&datatype.x.path)),
            ));
            commands.push(Arc::new(CommandX::Global(decl_box)));
            commands.push(Arc::new(CommandX::Global(decl_unbox)));
        }
        for datatype in datatypes.iter() {
            let nodes = crate::prelude::datatype_box_axioms(&path_to_air_ident_opaq(&datatype.x.path));
            let axioms = air::print_parse::nodes_to_commands(&nodes)
                .expect("internal error: malformed datatype axioms");
            commands.extend(axioms.iter().cloned());
        }
    }
    Arc::new(commands)
}
