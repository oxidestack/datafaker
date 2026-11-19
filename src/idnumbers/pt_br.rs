//! `net.datafaker.idnumbers.pt.br`: `IdNumberGeneratorPtBrUtil` and `DocumentFormatterUtil`, Brazilian CPF and
//! CNPJ numbers with their mod-11 check digits.

use crate::{Faker, Result};

/// `cnpj(faker, formatted, valid, multiBranch)`: eight random digits, the branch (`0001`, or 1–9998 zero-padded
/// when `multi_branch`) and two check digits; an invalid one is a random 14-digit number. Drawn again until its
/// validity is the one asked for.
pub fn cnpj(faker: &Faker, formatted: bool, valid: bool, multi_branch: bool) -> Result<String> {
    let random = faker.random();
    loop {
        let cnpj = if valid {
            let mut partial = String::new();
            for _ in 0..8 {
                partial.push_str(&random.next_int_bounded(9)?.to_string());
            }
            if multi_branch {
                partial.push_str(&format!("{:0>4}", random.next_int_between(1, 9999)?));
            } else {
                partial.push_str("0001");
            }
            let d1 = digit(weight(&partial, 9, 4, 12) + weight(&partial, 5, 0, 4));
            let d2 = digit(d1 * 2 + weight(&partial, 9, 5, 12) + weight(&partial, 6, 0, 5));
            format!("{partial}{d1}{d2}")
        } else {
            let low = random.next_int_bounded(1_000_000_000)? as i64;
            (low + (random.next_int_bounded(90)? as i64 + 10) * 1_000_000_000_000).to_string()
        };
        let result = if formatted { format_cnpj(&cnpj) } else { cnpj };
        if is_cnpj_valid(&result) == valid {
            return Ok(result);
        }
    }
}

/// `cpf(faker, formatted, valid)`: nine random digits and two check digits; an invalid one is a random 11-digit
/// number. Drawn again until its validity is the one asked for.
pub fn cpf(faker: &Faker, formatted: bool, valid: bool) -> Result<String> {
    let random = faker.random();
    loop {
        let cpf = if valid {
            let mut partial = String::new();
            for _ in 0..9 {
                partial.push_str(&random.next_int_bounded(9)?.to_string());
            }
            let d1 = digit(weight(&partial, 10, 0, 9));
            let d2 = digit(d1 * 2 + weight(&partial, 11, 0, 9));
            format!("{partial}{d1}{d2}")
        } else {
            let low = random.next_int_bounded(1_000_000_000)? as i64;
            (low + (random.next_int_bounded(90)? as i64 + 10) * 1_000_000_000).to_string()
        };
        let result = if formatted { format_cpf(&cpf) } else { cpf };
        if is_cpf_valid(&result) == valid {
            return Ok(result);
        }
    }
}

/// `isCNPJValid`: the digits' check digits are right (false for fewer than 12 digits).
pub fn is_cnpj_valid(cnpj: &str) -> bool {
    let digits = unmask(cnpj);
    if digits.len() < 12 {
        return false;
    }
    let d1 = digit(weight(&digits, 9, 4, 12) + weight(&digits, 5, 0, 4));
    let d2 = digit(d1 * 2 + weight(&digits, 9, 5, 12) + weight(&digits, 6, 0, 5));
    digits[12..].starts_with(&format!("{d1}{d2}"))
}

/// `isCPFValid`: the first nine digits followed by their check digits are all of the digits.
pub fn is_cpf_valid(cpf: &str) -> bool {
    let digits = unmask(cpf);
    if digits.len() < 9 {
        return false;
    }
    let d1 = digit(weight(&digits, 10, 0, 9));
    let d2 = digit(d1 * 2 + weight(&digits, 11, 0, 9));
    digits == format!("{}{d1}{d2}", &digits[..9])
}

/// `DocumentFormatterUtil.cnpj`: `##.###.###/####-##`.
pub fn format_cnpj(cnpj: &str) -> String {
    format!("{}.{}.{}/{}-{}", &cnpj[..2], &cnpj[2..5], &cnpj[5..8], &cnpj[8..12], &cnpj[12..])
}

/// `DocumentFormatterUtil.cpf`: `###.###.###-##`.
pub fn format_cpf(cpf: &str) -> String {
    format!("{}.{}.{}-{}", &cpf[..3], &cpf[3..6], &cpf[6..9], &cpf[9..])
}

/// `DocumentFormatterUtil.unmask`: the ASCII digits.
pub fn unmask(doc: &str) -> String {
    doc.chars().filter(char::is_ascii_digit).collect()
}

fn weight(num: &str, weight: i32, start: usize, end: usize) -> i32 {
    num.as_bytes()[start..end].iter().enumerate().map(|(i, b)| (*b as i32 - '0' as i32) * (weight - i as i32)).sum()
}

fn digit(verifying_digit: i32) -> i32 {
    let remainder = verifying_digit % 11;
    if remainder == 0 || remainder == 1 { 0 } else { 11 - remainder }
}
