use crate::RustEmitter;
use sifr_hir::HirExpr;

impl RustEmitter {
    /// Emit any walrus (named expression) assignments that need to be hoisted before a condition.
    pub(super) fn emit_walrus_hoists(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::WalrusExpr { name, value, ty } => {
                self.write_indent();
                self.write("let ");
                self.write(name);
                self.write(": ");
                self.write(&ty.rust_type());
                self.write(" = ");
                self.emit_expr(value);
                self.write(";\n");
            }
            HirExpr::Compare { left, comparators, .. } => {
                self.emit_walrus_hoists(left);
                for c in comparators {
                    self.emit_walrus_hoists(c);
                }
            }
            HirExpr::BoolOp { values, .. } => {
                for v in values {
                    self.emit_walrus_hoists(v);
                }
            }
            HirExpr::BinOp { left, right, .. } => {
                self.emit_walrus_hoists(left);
                self.emit_walrus_hoists(right);
            }
            _ => {}
        }
    }

    pub(super) fn emit_list_slice(
        &mut self,
        object: &HirExpr,
        start: Option<&HirExpr>,
        stop: Option<&HirExpr>,
        step: Option<&HirExpr>,
    ) {
        if let Some(step_expr) = step {
            // Step slicing
            self.write("{ let _v = &");
            self.emit_expr(object);
            self.write("; let _len = _v.len() as i64; let _step = ");
            self.emit_expr(step_expr);
            self.write("; ");

            // Resolve start
            self.write("let _start = ");
            if let Some(s) = start {
                self.write("{ let _s = ");
                self.emit_expr(s);
                self.write("; if _s < 0 { ((_len + _s).max(0)) as usize } else { (_s.min(_len)) as usize } }");
            } else {
                self.write("if _step > 0 { 0 } else { (_len - 1) as usize }");
            }
            self.write("; ");

            // Resolve stop
            self.write("let _stop = ");
            if let Some(e) = stop {
                self.write("{ let _e = ");
                self.emit_expr(e);
                self.write("; if _e < 0 { ((_len + _e).max(0)) as usize } else { (_e.min(_len)) as usize } }");
            } else {
                self.write("if _step > 0 { _len as usize } else { 0_usize.wrapping_sub(1) }");
            }
            self.write("; ");

            // Build result
            self.write("let mut _result = Vec::new(); ");
            self.write("if _step > 0 { let mut _i = _start; while _i < _stop { if let Some(_el) = _v.get(_i) { _result.push(_el.clone()); } _i += _step as usize; } }");
            self.write(" else { let mut _i = _start as i64; let _stop_i = _stop as i64; while _i > _stop_i { if _i >= 0 { if let Some(_el) = _v.get(_i as usize) { _result.push(_el.clone()); } } _i += _step; } }");
            self.write("; _result }");
        } else {
            // Simple slice without step
            self.write("{ let _v = &");
            self.emit_expr(object);
            self.write("; let _len = _v.len() as i64; ");

            self.write("let _start = ");
            if let Some(s) = start {
                self.write("{ let _s = ");
                self.emit_expr(s);
                self.write("; if _s < 0 { ((_len + _s).max(0)) as usize } else { (_s.min(_len)) as usize } }");
            } else {
                self.write("0_usize");
            }
            self.write("; ");

            self.write("let _stop = ");
            if let Some(e) = stop {
                self.write("{ let _e = ");
                self.emit_expr(e);
                self.write("; if _e < 0 { ((_len + _e).max(0)) as usize } else { (_e.min(_len)) as usize } }");
            } else {
                self.write("_len as usize");
            }
            self.write("; ");

            self.write("_v[_start.._stop].to_vec() }");
        }
    }

    pub(super) fn emit_string_slice(
        &mut self,
        object: &HirExpr,
        start: Option<&HirExpr>,
        stop: Option<&HirExpr>,
        step: Option<&HirExpr>,
    ) {
        if let Some(step_expr) = step {
            self.write("{ let _s: Vec<char> = ");
            self.emit_expr(object);
            self.write(".chars().collect(); let _len = _s.len() as i64; let _step = ");
            self.emit_expr(step_expr);
            self.write("; ");

            self.write("let _start = ");
            if let Some(s) = start {
                self.write("{ let _sv = ");
                self.emit_expr(s);
                self.write("; if _sv < 0 { ((_len + _sv).max(0)) as usize } else { (_sv.min(_len)) as usize } }");
            } else {
                self.write("if _step > 0 { 0 } else { (_len - 1) as usize }");
            }
            self.write("; ");

            self.write("let _stop = ");
            if let Some(e) = stop {
                self.write("{ let _ev = ");
                self.emit_expr(e);
                self.write("; if _ev < 0 { ((_len + _ev).max(0)) as usize } else { (_ev.min(_len)) as usize } }");
            } else {
                self.write("if _step > 0 { _len as usize } else { 0_usize.wrapping_sub(1) }");
            }
            self.write("; ");

            self.write("let mut _result = String::new(); ");
            self.write("if _step > 0 { let mut _i = _start; while _i < _stop { if let Some(&_ch) = _s.get(_i) { _result.push(_ch); } _i += _step as usize; } }");
            self.write(" else { let mut _i = _start as i64; let _stop_i = _stop as i64; while _i > _stop_i { if _i >= 0 { if let Some(&_ch) = _s.get(_i as usize) { _result.push(_ch); } } _i += _step; } }");
            self.write("; _result }");
        } else {
            self.write("{ let _s = &");
            self.emit_expr(object);
            self.write("; let _len = _s.chars().count() as i64; ");

            self.write("let _start = ");
            if let Some(s) = start {
                self.write("{ let _sv = ");
                self.emit_expr(s);
                self.write("; if _sv < 0 { ((_len + _sv).max(0)) as usize } else { (_sv.min(_len)) as usize } }");
            } else {
                self.write("0_usize");
            }
            self.write("; ");

            self.write("let _stop = ");
            if let Some(e) = stop {
                self.write("{ let _ev = ");
                self.emit_expr(e);
                self.write("; if _ev < 0 { ((_len + _ev).max(0)) as usize } else { (_ev.min(_len)) as usize } }");
            } else {
                self.write("_len as usize");
            }
            self.write("; ");

            self.write("_s.chars().skip(_start).take(_stop - _start).collect::<String>() }");
        }
    }
}
