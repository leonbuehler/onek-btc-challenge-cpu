// initially i wanted to write my own eliptic curve code to turn a privkey
// inti a pubkey, and i did. This is in fact, a working, self written
// implementation of privkey_to_pubkey
// i planned to optimize it to be fast, but never got around to do it
// so this is pretty slow as of now
use bnum::{BInt, BUint, cast::As};

const INT_SIZE: usize = 14;

const P: BInt<INT_SIZE> = BInt::parse_str_radix(
    "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F",
    16,
);

const G: Point = Point {
    x: BInt::parse_str_radix(
        "55066263022277343669578718895168534326250603453777594175500187360389116729240",
        10,
    ),
    y: BInt::parse_str_radix(
        "32670510020758816978083085130507043184471273380659243275938904335757337482424",
        10,
    ),
};

#[derive(PartialEq, Debug)]
pub struct Point {
    x: BInt<INT_SIZE>,
    y: BInt<INT_SIZE>,
}

// solution from https://rustp.org/number-theory/modular-inverse/
// fn get_modinv(mut a: BInt<INT_SIZE>) -> BInt<INT_SIZE> {
//     if a < BInt::ZERO {
//         a = ((a % P) + P) % P;
//     }
//     let mut x = P - BInt::TWO;
//     let mut ans = BInt::ONE;
//     if x <= BInt::ZERO {
//         return BInt::ONE;
//     }
//     loop {
//         if x == BInt::ONE {
//             return (ans * a) % P;
//         }
//         if x & BInt::ONE == BInt::ZERO {
//             a = (a * a) % P;
//             x >>= 1;
//             continue;
//         } else {
//             ans = (ans * a) % P;
//             x -= BInt::ONE;
//         }
//     }
// }
fn get_modinv(mut a: BInt<INT_SIZE>) -> BInt<INT_SIZE> {
    let mut m = P;
    if a < BInt::ZERO {
        a = ((a % P) + P) % P;
    }
    let mut y_prev = BInt::ZERO;
    let mut y = BInt::ONE;
    while a > BInt::ONE {
        let q = m / a;

        let y_before = y;
        y = y_prev - q * y;
        y_prev = y_before;

        let a_before = a;
        a = ((m % a) + a) % a;
        m = a_before;
    }
    return ((y % P) + P) % P;
}

fn get_double(point: &Point) -> Point {
    let slope = (point.x.pow(2) * BInt::THREE) * get_modinv(BInt::TWO * point.y);
    let slope = ((slope % P) + P) % P;

    let x = slope.pow(2) - (BInt::TWO * point.x);
    let x = ((x % P) + P) % P;

    let y = slope * (point.x - x) - point.y;
    let y = ((y % P) + P) % P;
    return Point { x, y };
}

fn get_sum(point1: Point, point2: Point) -> Point {
    if point1 == point2 {
        return get_double(&point1);
    }

    let slope = (point1.y - point2.y) * get_modinv(point1.x - point2.x);
    let slope = ((slope % P) + P) % P;

    let x = slope.pow(2) - point1.x - point2.x;
    let x = ((x % P) + P) % P;

    let y = (slope * (point1.x - x)) - point1.y;
    let y = ((y % P) + P) % P;

    return Point { x, y };
}

fn get_product(mut k: BInt<INT_SIZE>) -> Point {
    if k.is_negative() {
        // modify k
        k = -k;
        k += BInt::from(BInt::ONE << k.bits());
    }
    let k = k;
    let mut current = G;
    for n in (0..k.bits() - 1).rev() {
        current = get_double(&current);
        if k >> n & BInt::ONE == BInt::ONE {
            current = get_sum(current, G);
        }
    }
    return current;
}

pub fn priv_to_pubkey(k: BUint<4>) -> [u8; 33] {
    let k: BInt<INT_SIZE> = k.as_();
    let point = get_product(k);
    let mut ret = [0; 33];

    if point.y % BInt::TWO == BInt::ZERO {
        ret[0] = 2;
    } else {
        ret[0] = 3;
    }

    ret[1..33].copy_from_slice(&point.x.to_be_bytes()[80..]);

    return ret;
}

// fn fmt_bin(k: BInt<16>) {
//     println!("k: {k} in bin: ");
//     for digit in k.to_be_bytes() {
//         print!(" {digit:08b}");
//     }
//     println!();
// }
//
// fn fmt_bin4(k: BUint<4>) {
//     println!("k: {k} in bin: ");
//     for digit in k.to_be_bytes() {
//         print!(" {digit:08b}");
//     }
//     println!();
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_privkey_to_pubkey() {
        let private_key = "ef235aacf90d9f4aadd8c92e4b2562e1d9eb97f0df9ba3b508258739cb013db2";
        let correct_pubkey = "02b4632d08485ff1df2db55b9dafd23347d1c47a457072a1e87be26896549a8737";
        let key_int: BUint<4> = BUint::parse_str_radix(private_key, 16);
        let pubkey = priv_to_pubkey(key_int);
        let pubkey_hex = hex::encode(pubkey);
        assert_eq!(pubkey_hex, correct_pubkey);

        let private_key = "236fb6d5ad1f43";
        let correct_pubkey = "034af4b81f8c450c2c870ce1df184aff1297e5fcd54944d98d81e1a545ffb22596";
        let key_int: BUint<4> = BUint::parse_str_radix(private_key, 16);
        let pubkey = priv_to_pubkey(key_int);
        let pubkey_hex = hex::encode(pubkey);
        assert_eq!(pubkey_hex, correct_pubkey);
    }

    #[test]
    fn test_product() {
        let test_num1: BInt<INT_SIZE> = BInt::from(89);
        let correct_x =
            "95798783811477552110532139218095995588261607922943497599304995669953488256687";
        let correct_y =
            "62969615922034445131819050050848042428434876063790599311067024316063183397028";
        let correct = Point {
            x: BInt::parse_str_radix(correct_x, 10),
            y: BInt::parse_str_radix(correct_y, 10),
        };
        let product = get_product(test_num1);
        assert_eq!(product, correct);

        let test_num2: BInt<INT_SIZE> = BInt::from(-89);
        let correct_x =
            "59804711932513506784123989531696865543229948330373523204264618537577706349480";
        let correct_y =
            "89050872960406978327480571194996810247693376857669791125881153464831190071583";
        let correct = Point {
            x: BInt::parse_str_radix(correct_x, 10),
            y: BInt::parse_str_radix(correct_y, 10),
        };
        let product = get_product(test_num2);
        assert_eq!(product, correct);

        let test_num3: BInt<INT_SIZE> = BInt::parse_str_radix(
            "55066263022277343669578718895168534326250603453777594175500187360389116729240",
            10,
        );
        let correct_x =
            "41368939038460017089690463593392860417892426308765457203329747030588589193225";
        let correct_y =
            "35702972027818625020095973668955176075740885849864829235584237564223564379706";
        let correct = Point {
            x: BInt::parse_str_radix(correct_x, 10),
            y: BInt::parse_str_radix(correct_y, 10),
        };
        let product = get_product(test_num3);
        assert_eq!(product, correct);

        let test_num4: BInt<INT_SIZE> = BInt::parse_str_radix(
            "-55066263022277343669578718895168534326250603453777594175500187360389116729240",
            10,
        );
        let correct_x =
            "76544115982214074547071964238105055578602244481073870780352872352221713822661";
        let correct_y =
            "74427758258274835753677594796529988727611978346118166702307917374240029324247";
        let correct = Point {
            x: BInt::parse_str_radix(correct_x, 10),
            y: BInt::parse_str_radix(correct_y, 10),
        };
        let product = get_product(test_num4);
        assert_eq!(product, correct);
    }

    #[test]
    fn test_sum() {
        let test_point1 = Point {
            x: BInt::from(501),
            y: BInt::from(800),
        };
        let test_point2 = Point {
            x: BInt::from(930),
            y: BInt::from(272),
        };
        let correct_x =
            "13018045535556258657087862219911658279361714252350122584317716545267857151006";
        let correct_y =
            "42743461252372978829547596195742327386892106310347973343650478211385555262513";
        let correct = Point {
            x: BInt::parse_str_radix(correct_x, 10),
            y: BInt::parse_str_radix(correct_y, 10),
        };
        let sum = get_sum(test_point1, test_point2);
        assert_eq!(sum, correct);

        let test_point3 = Point {
            x: BInt::from(9309),
            y: BInt::from(-800),
        };
        let test_point4 = Point {
            x: BInt::from(-930),
            y: BInt::from(-272),
        };
        let correct_x =
            "24978885151650751176931327208523760981885185574042576129919050952254969946602";
        let correct_y =
            "32808814799247868874241764834084190518840676355931007736836319749266870449";
        let correct = Point {
            x: BInt::parse_str_radix(correct_x, 10),
            y: BInt::parse_str_radix(correct_y, 10),
        };
        let sum = get_sum(test_point3, test_point4);
        assert_eq!(sum, correct);
    }

    #[test]
    fn test_double() {
        let g_x_double = "C6047F9441ED7D6D3045406E95C07CD85C778E4B8CEF3CA7ABAC09B95C709EE5";
        let g_y_double = "1AE168FEA63DC339A3C58419466CEAEEF7F632653266D0E1236431A950CFE52A";
        let correct_double_g = Point {
            x: BInt::parse_str_radix(g_x_double, 16),
            y: BInt::parse_str_radix(g_y_double, 16),
        };
        let double_g = get_double(&G);
        assert_eq!(correct_double_g, double_g);

        let test_point1 = Point {
            x: BInt::from(500),
            y: BInt::from(800),
        };
        let correct_x = "8FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF70035441";
        let correct_y = "93FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF65E719E6";
        let correct_point1_double = Point {
            x: BInt::parse_str_radix(correct_x, 16),
            y: BInt::parse_str_radix(correct_y, 16),
        };
        let double_point1 = get_double(&test_point1);
        assert_eq!(double_point1, correct_point1_double);

        let test_point2 = Point {
            x: BInt::from(501),
            y: BInt::from(-800),
        };
        let correct_x = "907219652BD3C36113404EA4A8C154C985F06F694467381D7DBF487F3B237BE5";
        let correct_y = "977E8F190D173FB7A5F41AEF6F8F041461B6D43D03968D759EE88DF33E67AAF";
        let correct_point2_double = Point {
            x: BInt::parse_str_radix(correct_x, 16),
            y: BInt::parse_str_radix(correct_y, 16),
        };
        let double_point2 = get_double(&test_point2);
        assert_eq!(double_point2, correct_point2_double);
    }

    #[test]
    fn test_mod_inv() {
        let test_num = "-550";
        let correct_inv =
            "68843660328367992551832203814256265214580518155753571710732054491974888977516";
        let num: BInt<INT_SIZE> = BInt::parse_str_radix(test_num, 10);
        let modinv = get_modinv(num);
        assert_eq!(modinv.to_string(), correct_inv);

        let g_xcoord = "79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798";
        let correct_inv =
            "16048257703666452242803569546805946138055448571451565585555302070354637922038";
        let num: BInt<INT_SIZE> = BInt::parse_str_radix(g_xcoord, 16);
        let modinv = get_modinv(num);
        assert_eq!(modinv.to_string(), correct_inv);

        let test_num = "550";
        let correct_inv =
            "46948428908948202871738781194431642638689466509886992328725529515933945694147";
        let num: BInt<INT_SIZE> = BInt::parse_str_radix(test_num, 10);
        let modinv = get_modinv(num);
        assert_eq!(modinv.to_string(), correct_inv);

        let test_num = "551";
        let correct_inv =
            "33833986147382772165508037361885214454403752325169021434396862114833615938544";
        let num: BInt<INT_SIZE> = BInt::parse_str_radix(test_num, 10);
        let modinv = get_modinv(num);
        assert_eq!(modinv.to_string(), correct_inv);

        let g_x_neg =
            "-55066263022277343669578718895168534326250603453777594175500187360389116729240";
        let correct_inv =
            "99743831533649743180767415461881961715214536094188998453902281937554196749625";
        let num: BInt<INT_SIZE> = BInt::parse_str_radix(g_x_neg, 10);
        let modinv = get_modinv(num);
        assert_eq!(modinv.to_string(), correct_inv);
    }
}
