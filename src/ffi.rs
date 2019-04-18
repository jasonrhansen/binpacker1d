use crate::optimizer::{Optimizer, Solution, StockPiece};
use std::ptr;

#[no_mangle]
pub extern "C" fn opt_create(decimal_places: usize) -> *mut Optimizer {
    Box::into_raw(Box::new(Optimizer::new(decimal_places)))
}

#[no_mangle]
pub unsafe extern "C" fn opt_add_stock_length(
    opt: *mut Optimizer,
    stock_length: f64,
) -> *mut Optimizer {
    if opt.is_null() {
        return ptr::null_mut();
    }
    (&mut *opt).add_stock_length(stock_length)
}

#[no_mangle]
pub unsafe extern "C" fn opt_add_cut_piece_length(
    opt: *mut Optimizer,
    cut_piece_length: f64,
) -> *mut Optimizer {
    if opt.is_null() {
        return ptr::null_mut();
    }
    (&mut *opt).add_cut_length(cut_piece_length)
}

#[no_mangle]
pub unsafe extern "C" fn opt_set_blade_width(
    opt: *mut Optimizer,
    blade_width: f64,
) -> *mut Optimizer {
    if opt.is_null() {
        return ptr::null_mut();
    }
    (&mut *opt).set_blade_width(blade_width)
}

#[no_mangle]
pub unsafe extern "C" fn opt_set_random_seed(
    opt: *mut Optimizer,
    random_seed: u64,
) -> *mut Optimizer {
    if opt.is_null() {
        return ptr::null_mut();
    }
    (&mut *opt).set_random_seed(random_seed)
}

#[no_mangle]
pub unsafe extern "C" fn opt_run(opt: *mut Optimizer) -> *mut Solution {
    if opt.is_null() {
        return ptr::null_mut();
    }
    if let Ok(solution) = (&mut *opt).optimize() {
        Box::into_raw(Box::new(solution))
    } else {
        return ptr::null_mut();
    }
}

#[no_mangle]
pub unsafe extern "C" fn opt_destroy(opt: *mut Optimizer) {
    if !opt.is_null() {
        drop(Box::from_raw(opt));
    }
}

#[no_mangle]
pub unsafe extern "C" fn opt_result_get_num_stock_pieces(result: *mut Solution) -> i64 {
    if result.is_null() {
        return -1;
    }
    (&mut *result).repos_pieces.len() as i64
}

#[no_mangle]
pub unsafe extern "C" fn opt_result_get_stock_piece(
    result: *mut Solution,
    stock_piece_index: usize,
) -> *const StockPiece {
    if result.is_null() {
        return ptr::null_mut();
    }
    match (&mut *result).repos_pieces.get(stock_piece_index) {
        Some(repos_piece) => repos_piece,
        None => ptr::null(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn opt_result_destroy(result: *mut Solution) {
    if !result.is_null() {
        drop(Box::from_raw(result));
    }
}

#[no_mangle]
pub unsafe extern "C" fn stock_piece_get_length(stock_piece: *mut StockPiece) -> f64 {
    if stock_piece.is_null() {
        return -1.0;
    }
    (&mut *stock_piece).length
}

#[no_mangle]
pub unsafe extern "C" fn stock_piece_get_num_cut_pieces(stock_piece: *mut StockPiece) -> i64 {
    if stock_piece.is_null() {
        return -1;
    }
    (&mut *stock_piece).demand_pieces.len() as i64
}

#[no_mangle]
pub unsafe extern "C" fn stock_piece_get_cut_piece(
    stock_piece: *mut StockPiece,
    cut_piece_index: usize,
) -> CutPiece {
    if !stock_piece.is_null() {
        if let Some(demand_piece) = (&mut *stock_piece).demand_pieces.get(cut_piece_index) {
            return CutPiece {
                location: demand_piece.location,
                length: demand_piece.length,
            };
        };
    }

    return CutPiece {
        location: -1.0,
        length: -1.0,
    };
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CutPiece {
    pub location: f64,
    pub length: f64,
}
