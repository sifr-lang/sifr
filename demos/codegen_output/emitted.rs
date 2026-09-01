// src/main.rs
use ::sifr_runtime::SifrInt;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Timer {
    label: String,
}
impl Timer {
    const fn new(label: String) -> Self {
        let sifr_generated_field_value_39f7fcec8fcb623d_6c6162656c: String = label;
        Self {
            label: sifr_generated_field_value_39f7fcec8fcb623d_6c6162656c,
        }
    }
}
impl Timer {
    fn sifr_generated_enter__(&self) -> Timer {
        self.clone()
    }
}
impl Timer {
    const fn sifr_generated_exit__(&self) {}
}
impl ::std::fmt::Display for Timer {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Timer(label={})", self.label)
    }
}
pub trait Describable {
    fn describe(&self) -> String;
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Item {
    name: String,
}
impl Item {
    const fn new(name: String) -> Self {
        let sifr_generated_field_value_c4bcadba8e631b86_6e616d65: String = name;
        Self {
            name: sifr_generated_field_value_c4bcadba8e631b86_6e616d65,
        }
    }
}
impl Item {
    fn describe(&self) -> String {
        {
            let mut sifr_generated_concat: String = String::with_capacity(6usize);
            sifr_generated_concat.push_str("Item: ");
            sifr_generated_concat.push_str(self.name.clone().as_str());
            sifr_generated_concat
        }
    }
}
impl ::std::fmt::Display for Item {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Item(name={})", self.name)
    }
}
impl Describable for Item {
    fn describe(&self) -> String {
        Item::describe(self)
    }
}
fn main() {
    let nums: Vec<SifrInt> = vec![
        SifrInt::from_i64(5),
        SifrInt::from_i64(3),
        SifrInt::from_i64(1),
        SifrInt::from_i64(4),
        SifrInt::from_i64(2),
    ];
    let lo: Option<SifrInt> = nums.iter().cloned().min();
    let hi: Option<SifrInt> = nums.iter().cloned().max();
    if let Some(lo) = lo.clone() {
        println!("{lo}");
    }
    if let Some(hi) = hi.clone() {
        println!("{hi}");
    }
    let evens: Vec<SifrInt> = Box::new(nums.iter().cloned().filter(
        move |sifr_generated_filter_item| {
            let x = sifr_generated_filter_item.clone();
            &x.floor_mod_known_nonzero(&SifrInt::from_i64(2)) == &SifrInt::from_i64(0)
        },
    ))
    .collect::<Vec<_>>();
    println!("{evens:?}");
    let big: Vec<SifrInt> = {
        let mut sifr_generated_list_comp = Vec::new();
        for x in nums.iter().cloned() {
            if &x > &SifrInt::from_i64(2) {
                sifr_generated_list_comp.push(x);
            }
        }
        sifr_generated_list_comp
    };
    println!("{big:?}");
    let name: String = "World".to_string();
    let greeting: String = {
        let mut sifr_generated_concat: String = String::with_capacity(7usize + name.len() + 1usize);
        sifr_generated_concat.push_str("Hello, ");
        sifr_generated_concat.push_str(name.as_str());
        sifr_generated_concat.push('!');
        sifr_generated_concat
    };
    println!("{greeting}");
    {
        struct SifrGeneratedWithGuard0 {
            ctx: Timer,
        }
        impl Drop for SifrGeneratedWithGuard0 {
            fn drop(&mut self) {
                self.ctx.sifr_generated_exit__();
            }
        }
        let sifr_generated_ctx_0 = Timer::new("work".to_string());
        let sifr_generated_guard_0 = SifrGeneratedWithGuard0 {
            ctx: sifr_generated_ctx_0,
        };
        let _ = sifr_generated_guard_0.ctx.sifr_generated_enter__();
        println!("doing work inside with block");
    }
    let item: Item = Item::new("Widget".to_string());
    println!("{}", item.describe());
    println!("All codegen quality v2 improvements verified!");
}
