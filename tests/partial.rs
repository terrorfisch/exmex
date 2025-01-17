#[cfg(feature = "partial")]
use exmex::{parse, Differentiate, ExResult, Express, FlatEx};
#[cfg(feature = "partial")]
mod utils;
#[cfg(feature = "partial")]
use rand::{thread_rng, Rng};
#[cfg(feature = "partial")]
use smallvec::{smallvec, SmallVec};
#[cfg(feature = "partial")]
use std::ops::Range;
#[cfg(feature = "partial")]
use std::str::FromStr;
#[cfg(feature = "partial")]
#[test]
fn test_readme_partial() -> ExResult<()> {
    let expr = parse::<f64>("y*x^2")?;

    // d_x
    let dexpr_dx = expr.partial(0)?;
    assert_eq!(format!("{}", dexpr_dx), "({x}*2.0)*{y}");

    // d_xy
    let ddexpr_dxy = dexpr_dx.partial(1)?;
    assert_eq!(format!("{}", ddexpr_dxy), "{x}*2.0");
    let result = ddexpr_dxy.eval(&[2.0, f64::MAX])?;
    assert!((result - 4.0).abs() < 1e-12);

    // d_xyx
    let dddexpr_dxyx = ddexpr_dxy.partial(0)?;
    assert_eq!(format!("{}", dddexpr_dxyx), "2.0");
    let result = dddexpr_dxyx.eval(&[f64::MAX, f64::MAX])?;
    assert!((result - 2.0).abs() < 1e-12);

    // all in one
    let dddexpr_dxyx_iter = expr.partial_iter([0, 1, 0].iter())?;
    assert_eq!(format!("{}", dddexpr_dxyx_iter), "2.0");
    let result = dddexpr_dxyx_iter.eval(&[f64::MAX, f64::MAX])?;
    assert!((result - 2.0).abs() < 1e-12);

    Ok(())
}

#[cfg(feature = "partial")]
#[test]
fn test_partial() -> ExResult<()> {
    fn test_flatex(
        flatex: &FlatEx<f64>,
        var_idx: usize,
        n_vars: usize,
        random_range: Range<f64>,
        reference: fn(f64) -> f64,
    ) -> ExResult<()> {
        let mut rng = rand::thread_rng();
        assert!(flatex.partial(flatex.var_names().len()).is_err());

        // test flatex
        let deri = flatex.partial(var_idx)?;
        println!("flatex {}", flatex);
        println!("partial {}", deri);
        for _ in 0..3 {
            let vut = rng.gen_range(random_range.clone());
            let mut vars: SmallVec<[f64; 10]> = smallvec![0.0; n_vars];
            vars[var_idx] = vut;
            println!("value under test {}.", vut);
            utils::assert_float_eq_f64(deri.eval(&vars).unwrap(), reference(vut));
        }
        Ok(())
    }
    fn test(
        sut: &str,
        var_idx: usize,
        n_vars: usize,
        random_range: Range<f64>,
        reference: fn(f64) -> f64,
    ) -> ExResult<()> {
        println!("testing {}...", sut);
        let flatex = FlatEx::<f64>::from_str(sut)?;
        test_flatex(&flatex, var_idx, n_vars, random_range, reference)
    }

    let sut = "+x";
    let var_idx = 0;
    let n_vars = 1;
    let reference = |_: f64| 1.0;
    test(sut, var_idx, n_vars, -10000.0..10000.0, reference)?;

    let sut = "++x";
    let var_idx = 0;
    let n_vars = 1;
    let reference = |_: f64| 1.0;
    test(sut, var_idx, n_vars, -10000.0..10000.0, reference)?;

    let sut = "+-+x";
    let var_idx = 0;
    let n_vars = 1;
    let reference = |_: f64| -1.0;
    test(sut, var_idx, n_vars, -10000.0..10000.0, reference)?;

    let sut = "-x";
    let var_idx = 0;
    let n_vars = 1;
    let reference = |_: f64| -1.0;
    test(sut, var_idx, n_vars, -10000.0..10000.0, reference)?;

    let sut = "--x";
    let var_idx = 0;
    let n_vars = 1;
    let reference = |_: f64| 1.0;
    test(sut, var_idx, n_vars, -10000.0..10000.0, reference)?;

    let sut = "sin(sin(x))";
    let var_idx = 0;
    let n_vars = 1;
    let reference = |x: f64| x.sin().cos() * x.cos();
    test(sut, var_idx, n_vars, -10000.0..10000.0, reference)?;

    let sut = "sin(x)-cos(x)+a";
    let var_idx = 1;
    let n_vars = 2;
    let reference = |x: f64| x.cos() + x.sin();
    test(sut, var_idx, n_vars, -10000.0..10000.0, reference)?;
    let flatex_1 = FlatEx::<f64>::from_str(sut)?;
    let deri = flatex_1.partial(var_idx)?;
    let reference = |x: f64| -x.sin() + x.cos();
    test_flatex(&deri, var_idx, n_vars, -10000.0..10000.0, reference)?;
    let deri = deri.partial(var_idx)?;
    let reference = |x: f64| -x.cos() - x.sin();
    test_flatex(&deri, var_idx, n_vars, -10000.0..10000.0, reference)?;
    let deri = deri.partial(var_idx)?;
    let reference = |x: f64| x.sin() - x.cos();
    test_flatex(&deri, var_idx, n_vars, -10000.0..10000.0, reference)?;

    let sut = "sin(x)-cos(x)+tan(x)+a";
    let var_idx = 1;
    let n_vars = 2;
    let reference = |x: f64| x.cos() + x.sin() + 1.0 / (x.cos().powf(2.0));
    test(sut, var_idx, n_vars, -10000.0..10000.0, reference)?;

    let sut = "ln(v)*exp(v)+cos(x)+tan(x)+a";
    let var_idx = 1;
    let n_vars = 3;
    let reference = |x: f64| 1.0 / x * x.exp() + x.ln() * x.exp();
    test(sut, var_idx, n_vars, -10000.0..10000.0, reference)?;

    let sut = "a+z+sinh(v)/cosh(v)+b+tanh({v})";
    let var_idx = 2;
    let n_vars = 4;
    let reference = |x: f64| {
        (x.cosh() * x.cosh() - x.sinh() * x.sinh()) / x.cosh().powf(2.0)
            + 1.0 / (x.cosh().powf(2.0))
    };
    test(sut, var_idx, n_vars, -10000.0..10000.0, reference)?;

    let sut = "w+z+acos(v)+asin(v)+b+atan({v})";
    let var_idx = 1;
    let n_vars = 4;
    let reference = |x: f64| {
        1.0 / (1.0 - x.powf(2.0)).sqrt() - 1.0 / (1.0 - x.powf(2.0)).sqrt()
            + 1.0 / (1.0 + x.powf(2.0))
    };
    test(sut, var_idx, n_vars, -1.0..1.0, reference)?;

    let sut = "sqrt(var)*var^1.57";
    let var_idx = 0;
    let n_vars = 1;
    let reference = |x: f64| 1.0 / (2.0 * x.sqrt()) * x.powf(1.57) + x.sqrt() * 1.57 * x.powf(0.57);
    test(sut, var_idx, n_vars, 0.0..100.0, reference)?;
    Ok(())
}

#[cfg(feature = "partial")]
#[test]
fn test_partial_finite() -> ExResult<()> {
    fn test<'a>(sut: &str, range: Range<f64>) -> ExResult<()> {
        let flatex = exmex::parse::<f64>(sut)?;
        let n_vars = flatex.var_names().len();
        let step = 1e-5;
        let mut rng = thread_rng();

        let x0s: Vec<f64> = (0..n_vars).map(|_| rng.gen_range(range.clone())).collect();
        println!(
            "\n\n ---- test_partial_finite -\n checking derivatives at {:?} for {}",
            x0s, sut
        );
        for var_idx in 0..flatex.var_names().len() {
            let x1s: Vec<f64> = x0s
                .iter()
                .enumerate()
                .map(|(i, x0)| if i == var_idx { x0 + step } else { *x0 })
                .collect();

            let f0 = flatex.eval(&x0s)?;
            let f1 = flatex.eval(&x1s)?;
            let finite_diff = (f1 - f0) / step;
            let deri = flatex.clone().partial(var_idx)?;
            let deri_val = deri.eval(&x0s)?;
            println!(
                "test_partial_finite -\n {} (derivative)\n {} (finite diff)",
                deri_val, finite_diff
            );
            let msg = format!("sut {}, d_{} is {}", sut, var_idx, deri);
            println!("test_partial_finite - {}", msg);
            utils::assert_float_eq::<f64>(deri_val, finite_diff, 1e-5, 1e-3, msg.as_str());
        }
        Ok(())
    }
    test("sqrt(x)", 0.0..10000.0)?;
    test("asin(x)", -1.0..1.0)?;
    test("acos(x)", -1.0..1.0)?;
    test("atan(x)", -1.0..1.0)?;
    test("1/x", -10.0..10.0)?;
    test("x^x", 0.01..2.0)?;
    test("x^y", 4.036286084344371..4.036286084344372)?;
    test("z+sin(x)+cos(y)", -1.0..1.0)?;
    test("sin(cos(sin(z)))", -10.0..10.0)?;
    test("sin(x+z)", -10.0..10.0)?;
    test("sin(x-z)", -10.0..10.0)?;
    test("y-sin(x-z)", -10.0..10.0)?;
    test("(sin(x)^2)/x/4", -10.0..10.0)?;
    test("sin(y+x)/((x*2)/y)*(2*x)", -1.0..1.0)?;
    test("z*sin(x)+cos(y)^(1 + x^2)/(sin(z))", 0.01..1.0)?;
    test("ln(x^2)", 0.1..10.0)?;
    test("log2(x^2)", 0.1..10.0)?;
    test("log10(x^2)", 0.1..10.0)?;
    test("tan(x)", -1.0..1.0)?;
    test("tan(exp(x))", -1000.0..0.0)?;
    test("exp(y-x)", -1.0..1.0)?;
    test("sqrt(exp(y-x))", -1000.0..0.0)?;
    test("sin(sin(x+z))", -10.0..10.0)?;
    test("asin(sqrt(x+y))", 0.0..0.5)?;
    Ok(())
}

#[cfg(feature = "partial")]
#[test]
fn test_partial_iter() -> ExResult<()> {
    let sut = "a^2+b^2+c^2+x^2+y^2+z^2";
    let expr = exmex::parse::<f64>(sut)?;
    let deri = expr.partial_iter([0, 1, 2, 3, 4, 5].iter())?;
    utils::assert_float_eq::<f64>(
        0.0,
        deri.eval(&[7.0, 7.0, 7.0, 7.0, 7.0, 7.0])?,
        1e-12,
        1e-12,
        sut,
    );

    fn test3(sut: &str) -> ExResult<()> {
        let expr = exmex::parse::<f64>(sut)?;
        let deri = expr.partial_iter([0, 1, 2].iter())?;
        let mut deri_seq = expr;
        for i in 0..3 {
            deri_seq = deri_seq.partial(i)?;
        }
        let vals = [7.3, 4.2, 423.9];
        utils::assert_float_eq_f64(deri_seq.eval(&vals)?, deri.eval(&vals)?);
        Ok(())
    }

    test3("a^2*b^2*c^2")?;
    test3("a^2+b^2*c^2")?;
    test3("a^2-cos(sin(b^2))*c^3")?;
    test3("a^2*b^2/sin(c^2)")?;
    Ok(())
}

#[cfg(feature = "partial")]
#[test]
fn test_log() -> ExResult<()> {
    let test_vals = [0.001, 5.0, 10.0, 1000.0, 12341.2345];
    let deri_ln = exmex::parse::<f64>("ln(x)")?.partial(0)?;
    let deri_log = exmex::parse::<f64>("log(x)")?.partial(0)?;

    let expr = exmex::parse::<f64>("log10(x)")?;
    let deri = expr.partial(0)?;
    for v in test_vals {
        utils::assert_float_eq_f64(deri_ln.eval(&[v])? * 1.0 / 10.0f64.ln(), deri.eval(&[v])?);
        utils::assert_float_eq_f64(deri_log.eval(&[v])? * 1.0 / 10.0f64.ln(), deri.eval(&[v])?);
    }

    let expr = exmex::parse::<f64>("log2(x)")?;
    let deri = expr.partial(0)?;
    for v in test_vals {
        utils::assert_float_eq_f64(deri_ln.eval(&[v])? * 1.0 / 2.0f64.ln(), deri.eval(&[v])?);
    }
    Ok(())
}
