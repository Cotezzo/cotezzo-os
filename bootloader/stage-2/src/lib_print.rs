/* ==== PUBLIC PRINT MACRO ================================================== */
/*  Export print macro to print any ToString implementing type. */
#[macro_export]
macro_rules! print {
    ($($arg:expr),*) => {
        {
            $(
                let s = $arg.to_string();
                print_string(s);
                // for c in s.chars() { unsafe { _print_char(c as u8, 0); } }
            )*
        }
    };
}

/* ==== PRIVATE ASM PRINT - needs to be imported for the macro to work ====== */
/*  Reference a method not located in this Rust module, but search for it
    link-time. In this case, this is a global method declared in print.asm.
    Rust calls the method using the C calling convention (extern "C"). */
extern "C" { fn _print_char(char: u8, page: u8) -> (); }

/* Print each character in the array (ASCII string) using the ASM method. */
pub fn print_string(s: &str) {
    for c in s.as_bytes().iter() {
        unsafe { _print_char(*c, 0); }
    }
}

/* ==== PRIVATE TRAIT IMPLEMENTATION ======================================== */
/*  Define ToString trait so that we can implement a custom to_string function
    for each type we need to print with the print! macro.
    The trait is implement by defining a to_string method that returns a
    reference to a static string. */
pub trait ToString { fn to_string(&self) -> &'static str; }

/*  To implement the conversion to unsigned integer to string, we can use the
    same logic for each type, so a macro is used to generate the same impl
    for u8, u16, u32 and u64. */
#[macro_export]
macro_rules! to_string_impl_uint {
    // Match ty: type for which we are implementing the to_string method.
    // Match size: maximum size of the string representation of the number type.
    ($ty:ty, $size:expr) => {

        // Start code implementing ToString trait for the matched uint type
        impl ToString for $ty {
            fn to_string(&self) -> &'static str {

                // Static buffer to store ASCII representation without allocator
                static mut BUFFER: [u8; $size] = [0; $size];

                // Init buffer index to last char
                let mut i = $size - 1;

                // The number to print is self, the u8 we're writing the impl for
                let mut num = *self;

                unsafe {
                    loop {
                        // Get the rightmost number getting the /10 remainder
                        BUFFER[i] = b'0' + (num % 10) as u8;

                        // Actually divide to "shift" the number to the right
                        num /= 10;

                        // If there are no numbers left to print, exit
                        if num == 0 { break; }

                        // Change buffer index
                        i -= 1;
                    }

                    // Convert the u8 array to utf8 string (unchecked)
                    core::str::from_utf8_unchecked(&BUFFER[i..])
                }
            }
        }
    };
}

/*  Implement the ToString trait for the static &str type: return self. */
impl ToString for &'static str { fn to_string(&self) -> &'static str { *self } }

/*  Implement the ToString trait for u8 using the to_string_impl_uint macro. 
    Max size: 3 chars (0-255). */
to_string_impl_uint!(u8, 3);

/*  Implement the ToString trait for u16 using the to_string_impl_uint macro.
    Max size: 5 chars (0-65_535). */
to_string_impl_uint!(u16, 5);

/*  Implement the ToString trait for u32 using the to_string_impl_uint macro.
    Max size: 10 chars (0-4_294_967_295). */
to_string_impl_uint!(u32, 10);

/*  Implement the ToString trait for u64 using the to_string_impl_uint macro.
    Max size: 20 chars (0-18_446_744_073_709_551_615). */
to_string_impl_uint!(u64, 20);