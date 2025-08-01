use std::sync::Arc;

use indexmap::IndexSet;
use preset_env_base::{version::should_enable, Versions};
use rustc_hash::FxBuildHasher;
use swc_atoms::Atom;
use swc_ecma_ast::*;
use swc_ecma_visit::{noop_visit_type, Visit, VisitWith};

use self::builtin::BUILTINS;
pub(crate) use self::entry::Entry;

mod builtin;
mod data;
mod entry;

pub(crate) struct UsageVisitor {
    is_any_target: bool,
    target: Arc<Versions>,
    pub required: IndexSet<&'static str, FxBuildHasher>,
}

impl UsageVisitor {
    pub fn new(target: Arc<Versions>) -> Self {
        //        let mut v = Self { required: Vec::new() };
        //
        //
        //        let is_web_target = target
        //            .iter()
        //            .any(|(name, version)| name != "node" &&
        // version.is_some());
        //
        //        println!(
        //            "is_any_target={:?}\nis_web_target={:?}",
        //            is_any_target, is_web_target
        //        );
        //
        //        // Web default
        //        if is_any_target || is_web_target {
        //            v.add(&["web.timers", "web.immediate",
        // "web.dom.iterable"]);        }
        //        v

        //if target.is_any_target() || target.node.is_none() {
        //    v.add(&["web.timers", "web.immediate", "web.dom.iterable"]);
        //}

        Self {
            is_any_target: target.is_any_target(),
            target,
            required: Default::default(),
        }
    }

    /// Add imports
    fn add(&mut self, features: impl ExactSizeIterator<Item = &'static str>) {
        let UsageVisitor {
            is_any_target,
            target,
            ..
        } = self;

        self.required.extend(features.filter(|f| {
            if !*is_any_target {
                if let Some(v) = BUILTINS.get(*f) {
                    // Skip
                    if !should_enable(target, v, true) {
                        return false;
                    }
                }
            }

            true
        }));
    }

    fn add_property_deps_inner(&mut self, obj: Option<&Atom>, prop: &Atom) {
        if let Some(obj) = obj {
            if let Some(map) = data::static_properties_get(obj) {
                if let Some(features) = map.get(prop) {
                    self.add(features);
                }
            }
        }

        if let Some(features) = data::instance_properties_get(prop) {
            self.add(features);
        }
    }

    fn visit_object_pat_props(&mut self, obj: &Expr, props: &[ObjectPatProp]) {
        let obj = match obj {
            Expr::Ident(i) => Some(&i.sym),
            _ => None,
        };

        for p in props {
            match p {
                ObjectPatProp::KeyValue(KeyValuePatProp {
                    key: PropName::Ident(i),
                    ..
                }) => self.add_property_deps_inner(obj, &i.sym),
                ObjectPatProp::Assign(AssignPatProp { key, .. }) => {
                    self.add_property_deps_inner(obj, &key.sym)
                }

                _ => {}
            }
        }
    }
}

// TODO:
//     Program(path: NodePath) {
//      path.get("body").forEach(bodyPath => {
//        if (isPolyfillSource(getRequireSource(bodyPath))) {
//          console.warn(NO_DIRECT_POLYFILL_IMPORT);
//          bodyPath.remove();
//        }
//      });
//    },

/// Detects usage of types
impl Visit for UsageVisitor {
    noop_visit_type!(fail);

    fn visit_ident(&mut self, node: &Ident) {
        node.visit_children_with(self);

        if let Some(features) = data::builtin_types_get(&node.sym) {
            self.add(features);
        }
    }

    fn visit_var_declarator(&mut self, d: &VarDeclarator) {
        d.visit_children_with(self);

        if let Some(ref init) = d.init {
            if let Pat::Object(ref o) = d.name {
                self.visit_object_pat_props(init, &o.props)
            }
        } else if let Pat::Object(ref o) = d.name {
            self.visit_object_pat_props(&Ident::default().into(), &o.props)
        }
    }

    fn visit_assign_expr(&mut self, e: &AssignExpr) {
        e.visit_children_with(self);

        if let AssignTarget::Pat(AssignTargetPat::Object(o)) = &e.left {
            self.visit_object_pat_props(&e.right, &o.props)
        }
    }

    /// Detects usage of instance properties and static properties.
    ///
    ///  - `Array.from`
    fn visit_member_expr(&mut self, node: &MemberExpr) {
        node.obj.visit_with(self);
        if let MemberProp::Computed(c) = &node.prop {
            c.visit_with(self);
        }
        //enter(path: NodePath) {
        //    const { node } = path;
        //    const { object, property } = node;
        //
        //    // ignore namespace
        //    if (isNamespaced(path.get("object"))) return;
        //
        //    let evaluatedPropType = object.name;
        //    let propertyName = "";
        //    let instanceType = "";
        //
        //    if (node.computed) {
        //        if (t.isStringLiteral(property)) {
        //            propertyName = property.value;
        //        } else {
        //            const result = path.get("property").evaluate();
        //            if (result.confident && result.value) {
        //                propertyName = result.value;
        //            }
        //        }
        //    } else {
        //        propertyName = property.name;
        //    }
        //
        //    if (path.scope.getBindingIdentifier(object.name)) {
        //        const result = path.get("object").evaluate();
        //        if (result.value) {
        //            instanceType = getType(result.value);
        //        } else if (result.deopt && result.deopt.isIdentifier()) {
        //            evaluatedPropType = result.deopt.node.name;
        //        }
        //    }
        //
        //    if (has(STATIC_PROPERTIES, evaluatedPropType)) {
        //        const BuiltInProperties = STATIC_PROPERTIES[evaluatedPropType];
        //        if (has(BuiltInProperties, propertyName)) {
        //            const StaticPropertyDependencies =
        // BuiltInProperties[propertyName];
        // this.addUnsupported(StaticPropertyDependencies);        }
        //    }
        //
        //    if (has(INSTANCE_PROPERTIES, propertyName)) {
        //        let InstancePropertyDependencies = INSTANCE_PROPERTIES[propertyName];
        //        if (instanceType) {
        //            InstancePropertyDependencies =
        // InstancePropertyDependencies.filter(                module =>
        // module.includes(instanceType),            );
        //        }
        //        this.addUnsupported(InstancePropertyDependencies);
        //    }
        //},
        //
        //// Symbol.match
        //exit(path: NodePath) {
        //    const { name } = path.node.object;
        //
        //    if (!has(BUILT_INS, name)) return;
        //    if (path.scope.getBindingIdentifier(name)) return;
        //
        //    const BuiltInDependencies = BUILT_INS[name];
        //    this.addUnsupported(BuiltInDependencies);
        //},

        match &node.prop {
            MemberProp::Ident(i) => {
                //
                if let Some(imports) = data::instance_properties_get(&i.sym) {
                    self.add(imports);
                }
            }
            MemberProp::Computed(ComputedPropName { expr, .. }) => {
                if let Expr::Lit(Lit::Str(Str { value, .. })) = &**expr {
                    if let Some(imports) = data::instance_properties_get(value) {
                        self.add(imports);
                    }
                }
            }
            _ => {}
        }
        if let Expr::Ident(obj) = &*node.obj {
            if let Some(props) = data::static_properties_get(&obj.sym) {
                match &node.prop {
                    MemberProp::Computed(ComputedPropName { expr, .. }) => {
                        if let Expr::Lit(Lit::Str(Str { value, .. })) = &**expr {
                            if let Some(imports) = data::instance_properties_get(value) {
                                self.add(imports);
                            }
                        }
                    }

                    MemberProp::Ident(ref p) => {
                        if let Some(imports) = props.get(&p.sym) {
                            self.add(imports);
                        }
                    }

                    _ => {}
                }
            }
        }
    }

    // maybe not needed?
    fn visit_super_prop_expr(&mut self, node: &SuperPropExpr) {
        if let SuperProp::Computed(c) = &node.prop {
            c.visit_with(self);
        }
    }

    ///
    /// - `arr[Symbol.iterator]()`
    fn visit_call_expr(&mut self, e: &CallExpr) {
        e.visit_children_with(self);

        if match &e.callee {
            Callee::Expr(callee) => matches!(&**callee, Expr::Member(MemberExpr {
                    prop: MemberProp::Computed(ComputedPropName { expr, .. }),
                    ..
                }) if is_symbol_iterator(expr)),
            _ => false,
        } {
            self.add(["web.dom.iterable"].iter().copied())
        }
    }

    ///
    /// - `Symbol.iterator in arr`
    fn visit_bin_expr(&mut self, e: &BinExpr) {
        e.visit_children_with(self);

        match e.op {
            op!("in") if is_symbol_iterator(&e.left) => {
                self.add(["web.dom.iterable"].iter().copied())
            }
            _ => {}
        }
    }

    ///
    /// - `yield*`
    fn visit_yield_expr(&mut self, e: &YieldExpr) {
        e.visit_children_with(self);

        if e.delegate {
            self.add(["web.dom.iterable"].iter().copied())
        }
    }
}

fn is_symbol_iterator(e: &Expr) -> bool {
    match e {
        Expr::Member(MemberExpr { obj, prop, .. }) if prop.is_ident_with("iterator") => {
            obj.is_ident_ref_to("Symbol")
        }
        _ => false,
    }
}
