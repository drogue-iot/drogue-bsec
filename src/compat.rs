/// Exported pow(double, double)
#[no_mangle]
pub extern "C" fn pow(x: cty::c_double, y: cty::c_double) -> cty::c_double {
    libm::pow(x, y)
}

/// Exported powf(float, float)
#[no_mangle]
pub extern "C" fn powf(x: cty::c_float, y: cty::c_float) -> cty::c_float {
    libm::powf(x, y)
}

/// Exported sqrt(double)
#[no_mangle]
pub extern "C" fn sqrt(x: cty::c_double) -> cty::c_double {
    libm::sqrt(x)
}

/// Exported sqrtf(float)
#[no_mangle]
pub extern "C" fn sqrtf(x: cty::c_float) -> cty::c_float {
    libm::sqrtf(x)
}

/// Exported ceil(double)
#[no_mangle]
pub extern "C" fn ceil(x: cty::c_double) -> cty::c_double {
    libm::ceil(x)
}

/// Exported floor(double)
#[no_mangle]
pub extern "C" fn floor(x: cty::c_double) -> cty::c_double {
    libm::floor(x)
}

/// Exported ceilf(float)
#[no_mangle]
pub extern "C" fn ceilf(x: cty::c_float) -> cty::c_float {
    libm::ceilf(x)
}

/// Exported floorf(float)
#[no_mangle]
pub extern "C" fn floorf(x: cty::c_float) -> cty::c_float {
    libm::floorf(x)
}

/// Exported logf(float)
#[no_mangle]
pub extern "C" fn logf(x: cty::c_float) -> cty::c_float {
    libm::logf(x)
}

/// Exported log10f(float)
#[no_mangle]
pub extern "C" fn log10f(x: cty::c_float) -> cty::c_float {
    libm::log10f(x)
}

/// Exported sinf(float)
#[no_mangle]
pub extern "C" fn sinf(x: cty::c_float) -> cty::c_float {
    libm::sinf(x)
}

/// Exported fabsf(float)
#[no_mangle]
pub extern "C" fn fabsf(x: cty::c_float) -> cty::c_float {
    libm::fabsf(x)
}

/// Exported fabs(double)
#[no_mangle]
pub extern "C" fn fabs(x: cty::c_double) -> cty::c_double {
    libm::fabs(x)
}

/// Exported expf(float)
#[no_mangle]
pub extern "C" fn expf(x: cty::c_float) -> cty::c_float {
    libm::expf(x)
}

/// Exported roundf(float)
#[no_mangle]
pub extern "C" fn roundf(x: cty::c_float) -> cty::c_float {
    libm::roundf(x)
}

/// Exported fminf(float)
#[no_mangle]
pub extern "C" fn fminf(x: cty::c_float, y: cty::c_float) -> cty::c_float {
    libm::fminf(x, y)
}

/// Exported fmaxf(float)
#[no_mangle]
pub extern "C" fn fmaxf(x: cty::c_float, y: cty::c_float) -> cty::c_float {
    libm::fmaxf(x, y)
}
