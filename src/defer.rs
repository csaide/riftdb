// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0-only

pub(crate) struct ScopeCall<F: FnOnce()> {
    pub(crate) c: Option<F>,
}

impl<F: FnOnce()> Drop for ScopeCall<F> {
    fn drop(&mut self) {
        self.c.take().unwrap()()
    }
}

macro_rules! expr {
    ($e: expr) => {
        $e
    };
}

macro_rules! defer {
    ($($data: tt)*) => (
        let _scope_call = crate::defer::ScopeCall {
            c: Some(|| -> () { crate::defer::expr!({ $($data)* }) })
        };
    )
}

pub(crate) use defer;
pub(crate) use expr;
