use alloc::vec::Vec;

pub fn tag(tag: &[u8]) -> impl Fn(&[u8]) -> Result<(&[u8], &[u8]), &'static str> + '_ {
    move |input: &[u8]| {
        if tag.len() > input.len() {
            return Err("trying to tag too many bytes");
        }
        for i in 0..tag.len() {
            if tag[i] != input[i] {
                return Err("did not match tag");
            }
        }
        Ok((&input[tag.len()..], &input[..tag.len()]))
    }
}

pub fn take(num: usize) -> impl Fn(&[u8]) -> Result<(&[u8], &[u8]), &'static str> {
    move |input: &[u8]| {
        if num > input.len() {
            return Err("trying to take too many bytes");
        }
        Ok((&input[num..], &input[..num]))
    }
}

pub fn many_n<'a, T>(
    n: usize,
    f: impl Fn(&'a [u8]) -> Result<(&'a [u8], T), &'static str>,
) -> impl Fn(&'a [u8]) -> Result<(&'a [u8], Vec<T>), &'static str> {
    move |input: &[u8]| {
        let mut v = vec![];
        let mut ip = input;
        loop {
            if n == v.len() {
                break;
            }
            match f(ip) {
                Ok((input, item)) => {
                    v.push(item);
                    ip = input;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok((ip, v))
    }
}
