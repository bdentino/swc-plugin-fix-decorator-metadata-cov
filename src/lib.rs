// use swc_core::common::comments::{Comment, CommentKind, Comments};
use swc_core::common::Span;
use swc_core::common::Spanned;
use swc_core::common::DUMMY_SP;
use swc_core::ecma::ast::{/*CondExpr,*/ UnaryExpr};
use swc_core::ecma::visit::{VisitMut, VisitMutWith};
use swc_core::ecma::{
    ast::{ArrayLit, CallExpr, Callee, Expr, ExprOrSpread, Ident, Program},
    visit::{as_folder, FoldWith},
};
// use swc_core::plugin::proxies::PluginCommentsProxy;
use swc_core::plugin::{plugin_transform, proxies::TransformPluginProgramMetadata};

pub struct TransformVisitor {
    // comments: Option<PluginCommentsProxy>,
}

impl VisitMut for TransformVisitor {
    fn visit_mut_call_expr(&mut self, n: &mut CallExpr) {
        if let Callee::Expr(ref callee_expr) = n.callee {
            if let Expr::Ident(Ident { sym, .. }) = &**callee_expr {
                // find the _ts_metadata(...) call
                if sym.as_ref() == "_ts_metadata" {
                    // get a reference to the 2nd argument
                    if let Some(Expr::Array(ArrayLit { elems, .. })) =
                        n.args.get_mut(1).map(|arg| &mut *arg.expr)
                    {
                        // get the first element in the array
                        if let Some(Some(ExprOrSpread { expr, .. })) = elems.get_mut(0) {
                            let span = &mut DUMMY_SP.clone();

                            // continue with element as ternary expression
                            if let Expr::Cond(cond_expr) = &mut **expr {
                                // Reset span on arg in ternary test
                                let test = &mut cond_expr.test;
                                if let Expr::Bin(binexpr) = &mut **test {
                                    if let Expr::Unary(UnaryExpr { arg, .. }) = &mut *binexpr.left {
                                        if let Expr::Ident(argident) = &mut **arg {
                                            *span = Span { ..argident.span() };
                                            *argident = Ident {
                                                span: DUMMY_SP,
                                                ..argident.clone()
                                            };
                                        }
                                    }
                                }

                                // Reset span on ternary alt
                                let alt = &mut cond_expr.alt;
                                if let Expr::Ident(altident) = &mut **alt {
                                    *altident = Ident {
                                        span: DUMMY_SP,
                                        ..altident.clone()
                                    };
                                }
                            }

                            /*
                             * I first tried to make this work by inserting an ignore
                             * comment at the start of the ternary expression. This
                             * code does insert the comment in the expected position,
                             * but I found that this does not fix the coverage complaint
                             * because when v8 coverage output is mapped for reporting,
                             * ignore hints are parsed from the original source. So really
                             * the only viable solution is to reset the spans on ternary nodes
                             * so that the coverage gaps over these nodes cannot be mapped back
                             * to a valid token in the original source.
                             */
                            // if let Expr::Cond(cond_expr) = &mut **expr {
                            //     *cond_expr = CondExpr {
                            //         span: *span,
                            //         ..cond_expr.clone()
                            //     }
                            //     .into();
                            //     let comment = Comment {
                            //         kind: CommentKind::Block,
                            //         text: " c8 ignore next ".into(),
                            //         span: *span,
                            //     };
                            //     self.comments.add_leading(span.lo(), comment);
                            // }
                        }
                    }
                }
            }
        }
        n.visit_mut_children_with(self);
    }
}

#[plugin_transform]
pub fn process(program: Program, _metadata: TransformPluginProgramMetadata) -> Program {
    let visitor = TransformVisitor {
        // comments: _metadata.comments,
    };
    program.fold_with(&mut as_folder(visitor))
}
