//! `StrictMath` functions, ported from fdlibm 5.3 as the JDK's `FdLibm` class does.

// fdlibm's constants are written with the digits of fdlibm's source, more than an f64 holds, so each can be checked
// against the C and Java sources digit for digit; each rounds to the same f64 a shorter literal would.
#![allow(clippy::excessive_precision)]

const LN2_HI: f64 = 6.931_471_803_691_238_164_90e-01;
const LN2_LO: f64 = 1.908_214_929_270_587_700_02e-10;
const TWO54: f64 = 1.801_439_850_948_198_400_00e+16;
const LG1: f64 = 6.666_666_666_666_735_130e-01;
const LG2: f64 = 3.999_999_999_940_941_908e-01;
const LG3: f64 = 2.857_142_874_366_239_149e-01;
const LG4: f64 = 2.222_219_843_214_978_396e-01;
const LG5: f64 = 1.818_357_216_161_805_012e-01;
const LG6: f64 = 1.531_383_769_920_937_332e-01;
const LG7: f64 = 1.479_819_860_511_658_591e-01;

fn hi(x: f64) -> i32 {
    (x.to_bits() >> 32) as i32
}

fn lo(x: f64) -> u32 {
    x.to_bits() as u32
}

fn with_hi(x: f64, hi: i32) -> f64 {
    f64::from_bits(((hi as u32 as u64) << 32) | (x.to_bits() & 0xffff_ffff))
}

/// `StrictMath.log(x)` (`__ieee754_log`).
pub fn log(mut x: f64) -> f64 {
    let mut hx = hi(x);
    let lx = lo(x);
    let mut k: i32 = 0;
    if hx < 0x0010_0000 {
        if ((hx & 0x7fff_ffff) as u32 | lx) == 0 {
            return f64::NEG_INFINITY;
        }
        if hx < 0 {
            return f64::NAN;
        }
        k -= 54;
        x *= TWO54;
        hx = hi(x);
    }
    if hx >= 0x7ff0_0000 {
        return x + x;
    }
    k += (hx >> 20) - 1023;
    hx &= 0x000f_ffff;
    let i = (hx + 0x95f64) & 0x0010_0000;
    x = with_hi(x, hx | (i ^ 0x3ff0_0000));
    k += i >> 20;
    let f = x - 1.0;
    if (0x000f_ffff & (2 + hx)) < 3 {
        if f == 0.0 {
            if k == 0 {
                return 0.0;
            }
            let dk = k as f64;
            return dk * LN2_HI + dk * LN2_LO;
        }
        let r = f * f * (0.5 - 0.333_333_333_333_333_33 * f);
        if k == 0 {
            return f - r;
        }
        let dk = k as f64;
        return dk * LN2_HI - ((r - dk * LN2_LO) - f);
    }
    let s = f / (2.0 + f);
    let dk = k as f64;
    let z = s * s;
    let mut i = hx - 0x6147a;
    let w = z * z;
    let j = 0x6b851 - hx;
    let t1 = w * (LG2 + w * (LG4 + w * LG6));
    let t2 = z * (LG1 + w * (LG3 + w * (LG5 + w * LG7)));
    i |= j;
    let r = t2 + t1;
    if i > 0 {
        let hfsq = 0.5 * f * f;
        if k == 0 {
            f - (hfsq - s * (hfsq + r))
        } else {
            dk * LN2_HI - ((hfsq - (s * (hfsq + r) + dk * LN2_LO)) - f)
        }
    } else if k == 0 {
        f - s * (f - r)
    } else {
        dk * LN2_HI - ((s * (f - r) - dk * LN2_LO) - f)
    }
}
