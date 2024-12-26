//! A modified rip off the rust-vcd writer library,
//! Prints a VCD file data to stdout, in an 'immediate-mode' fashion.

use alloc::borrow::ToOwned;

use crate::alloc::collections::BTreeMap;
use crate::alloc::vec::Vec;
use crate::alloc::{
    format,
    string::{String, ToString},
};
use crate::println;
use crate::{Duration, Instant};

// enum for timescale values
// The third line identifies the timescale. The timescale includes a time number (1, 10, or 100) followed by a unit (s, ms, us, ns, ps, or fs). Time integers within the file may then be multiplied by this unit to turn them into engineering units in a display.
#[derive(Clone, Copy)]
pub enum TimeScale {
    _1 = 1,
    _10 = 10,
    _100 = 100,
}

#[derive(Copy, Clone)]
pub enum TimeUnit {
    S,
    Ms,
    Us,
    Ns,
}

// a simple struct that holds the metadata (non-data) of the VCD file
#[derive(Clone)]
pub struct VcdHeaderMeta {
    pub version: String,
    pub creation_date: Instant,
    pub timescale: (TimeScale, TimeUnit),
}

impl Into<u64> for TimeScale {
    fn into(self) -> u64 {
        match self {
            TimeScale::_1 => 1,
            TimeScale::_10 => 10,
            TimeScale::_100 => 100,
        }
    }
}

impl ToString for TimeScale {
    fn to_string(&self) -> String {
        match self {
            TimeScale::_1 => "1".to_string(),
            TimeScale::_10 => "10".to_string(),
            TimeScale::_100 => "100".to_string(),
        }
    }
}

impl ToString for TimeUnit {
    fn to_string(&self) -> String {
        match self {
            TimeUnit::S => "s".to_string(),
            TimeUnit::Ms => "ms".to_string(),
            TimeUnit::Us => "us".to_string(),
            TimeUnit::Ns => "ns".to_string(),
        }
    }
}

impl ToString for VcdHeaderMeta {
    fn to_string(&self) -> String {
        // format creation_date like
        // $date Wed Jun  7 11:35:32 2017$end
        format!(
            "$version {} $end\n$timescale {}{} $end\n",
            self.version,
            self.timescale.0.to_string(),
            self.timescale.1.to_string(),
        )
    }
}

type ScopeName = String;
type VarName = String;
type VarIdent = String;

#[derive(Clone)]
pub enum VarType {
    Wire,
    Reg,
    Parameter,
    Event,
    Integer,
    Real,
    Supply0,
    Supply1,
    Time,
    Tri,
    Triand,
    Trior,
    Trireg,
    Tri0,
    Tri1,
    Wand,
    Wor,
}

impl ToString for VarType {
    fn to_string(&self) -> String {
        match self {
            VarType::Wire => "wire".to_string(),
            VarType::Reg => "reg".to_string(),
            VarType::Parameter => "parameter".to_string(),
            VarType::Event => "event".to_string(),
            VarType::Integer => "integer".to_string(),
            VarType::Real => "real".to_string(),
            VarType::Supply0 => "supply0".to_string(),
            VarType::Supply1 => "supply1".to_string(),
            VarType::Time => "time".to_string(),
            VarType::Tri => "tri".to_string(),
            VarType::Triand => "triand".to_string(),
            VarType::Trior => "trior".to_string(),
            VarType::Trireg => "trireg".to_string(),
            VarType::Tri0 => "tri0".to_string(),
            VarType::Tri1 => "tri1".to_string(),
            VarType::Wand => "wand".to_string(),
            VarType::Wor => "wor".to_string(),
        }
    }
}

/* Variable types from the manual
Syntax:
    $var var_type size identifier reference $end
        var_type = event | integer | parameter | real | reg |
                   supply0 | supply1 | time | tri | triand |
                   trior | trireg | tri0 | tri1 | wand | wire | wor
        size = decimal value of number of bits.
        identifier = name of the variable in printable ASCII characters.
        reference = bit or vector name mapped to the identifier
Examples:
    $var wire 1 * en_q $end
    $var reg 8 ( data_q[7:0] $end
*/
/*
Variables themselves are declared on lines between $var and $end tags. Four tokens are used, between these two flags, to define any variable, as shown below:
    $var vary_type size identifier_code reference $end
The first token, var_type specifies the type of variable. The standard allows many different variable types, although I’ve only ever used wire. Other types that might be useful include parameter, and reg, although the standard identifies many more types.
The second token, size specifies the number of bits this value will contain.
The third token is perhaps the most cryptic, although it need not be. This is the identifer_code assigned to this particular variable. This is a printable character, or string of printable characters, used to identify the variable during the data section of the file. We’ll come back to this in a moment.
The last part of the $var line is the reference. This is the variable name the user has given to the trace. If the variable had a width, it would then be followed by something like [MSB:LSB]. For example, a four bit trace i_button could have the reference of i_button[3:0].
*/
#[derive(Clone)]
pub struct VcdVar {
    vtype: VarType,
    vsize: u32,
    ident: VarIdent,
    reference: VarName,
}

// struct for scopes
#[derive(Clone)]
pub struct VcdScopeModule {
    pub name: ScopeName,
    child_scopes: BTreeMap<String, VcdScopeModule>,
    child_vars: BTreeMap<String, VcdVar>,
}

/*
From the end of the header to the end of the file is the data section. This section contains two types of lines: simulation time lines and value change lines.
Simulation time lines start with a # and a time value. That’s it. For example,
#295
specifies that the following changes happen at 295 time units. Exactly how much time this references depends upon the $timescale command in the header. Further, the simulation time is an unsigned number. Negative numbers are not allowed, and will really mess up your VCD file. (I know … I’ve tried.)
Value change lines contain the value the variable is taking on, followed by the identifier code for the variable that was assigned in the header. These lines are only necessary any time the value in question changes.
For single bit values, the value in a value change line consists of a 0, 1, x, or z followed by the identifier code that was assigned to this value in the header. For multibit values, a b precedes all of the bits. If not all of the bits are given, then the value is left-extended in an unsigned fashion.
As an example, if J is defined in the header to reference i_clk, then
0J
specifies that the clock is now set to zero. Likewise if # is assigned to the 8-bit data value i_data[7:0], then this value can be set with
b01000101#
*/
// VcdData is created form a VcdVar method
impl VcdVar {
    pub fn new(name: &str, ident: &str, vtype: VarType, vsize: u32) -> Self {
        VcdVar {
            vtype,
            vsize,
            ident: ident.to_string(),
            reference: name.to_string(),
        }
    }

    pub fn set_vtype(&mut self, vtype: VarType) -> &mut Self {
        self.vtype = vtype;
        self
    }
    pub fn set_vsize(&mut self, size: u32) -> &mut Self {
        self.vsize = size;
        self
    }
    pub fn set_ident(&mut self, code: &str) -> &mut Self {
        self.ident = code.into();
        self
    }
    pub fn set_reference(&mut self, name: &str) -> &mut Self {
        self.reference = name.into();
        self
    }

    // print some data for this variable
    /*
    Value Changes
        The value change section is a listing of variables with their value. If the variable
        is a scalar (1 bit) then the value change format will be the value (0,1,x,z)
        followed by the identifier without any space characters in between. If the
        variable is a vector then value change begins with either the b (binary) or r
        (real_number) character followed by the vector value without any space
        character between the b or r and the vector value. Then a mandatory white
        space character followed by the identifier.
        Syntax:
            value+identifider
            b|r+value identifier
        Examples:
            1*
            0(
            b1010 &
            b1101 ^
    */
    pub fn print_data(&self, data: u32) -> String {
        match self.vtype {
            VarType::Real => {
                format!("r{} {}", data, self.ident,)
            }
            _ => {
                match self.vsize {
                    1 => {
                        format!(
                            "{:0width$b}{}",
                            data,
                            self.ident,
                            width = self.vsize as usize
                        )
                    }
                    _ => {
                        // multi-bit is prefixed with b,
                        // and a space is added to separate the data
                        format!("b{:b} {}", data, self.ident,)
                    }
                }
            }
        }
    }

    pub fn print_highz(&self) -> String {
        format!("z{}", self.ident)
    }

    pub fn print_x(&self) -> String {
        format!("x{}", self.ident)
    }

    pub fn print_1(&self) -> String {
        format!("1{}", self.ident)
    }

    pub fn print_0(&self) -> String {
        format!("0{}", self.ident)
    }
}

impl ToString for VcdVar {
    fn to_string(&self) -> String {
        format!(
            "$var {} {} {} {} $end\n",
            self.vtype.to_string(),
            self.vsize,
            self.ident,
            self.reference,
        )
    }
}

impl ToString for VcdScopeModule {
    fn to_string(&self) -> String {
        let mut s = format!("$scope module {} $end\n", self.name);
        for (name, scope) in &self.child_scopes {
            s.push_str(&scope.to_string());
        }
        for (name, var) in &self.child_vars {
            s.push_str(&var.to_string());
        }
        s.push_str("$upscope $end\n");
        s
    }
}

// top-level struct
#[derive(Clone)]
pub struct VcdFile {
    header: VcdHeaderMeta,
    root_scope: VcdScopeModule,
}

impl VcdFile {
    pub fn set_timescale(&mut self, scale: TimeScale, unit: TimeUnit) -> &mut Self {
        self.header.timescale = (scale, unit);
        self
    }

    pub fn set_version(&mut self, version: &str) -> &mut Self {
        self.header.version = version.to_string();
        self
    }

    pub fn root_scope(&mut self) -> &mut VcdScopeModule {
        &mut self.root_scope
    }

    pub fn root_scope_ref(&self) -> &VcdScopeModule {
        &self.root_scope
    }

    pub fn get_timescale(&self) -> TimeScale {
        self.header.clone().timescale.0
    }

    pub fn get_timeunit(&self) -> TimeUnit {
        self.header.clone().timescale.1
    }

    // print timestamp
    pub fn print_timestamp(now: Instant, unit: TimeUnit, scale: TimeScale) -> String {
        let dt = now.elapsed();
        // format depending on the timescale
        let ts = match unit {
            TimeUnit::S => dt.as_secs(),
            TimeUnit::Ms => dt.as_millis(),
            TimeUnit::Us => dt.as_micros(),
            TimeUnit::Ns => dt.as_micros() * 1000,
        };
        // divide by the timescale
        let n: u64 = scale.into();
        let tn = ts / n;
        format!("#{}", tn)
    }
}

impl ToString for VcdFile {
    fn to_string(&self) -> String {
        let mut s = self.header.to_string();
        s.push_str(&self.root_scope.to_string());
        s.push_str("$enddefinitions $end\n");
        s
    }
}

impl Default for VcdHeaderMeta {
    fn default() -> Self {
        VcdHeaderMeta {
            version: "LTT v0.1".to_string(),
            creation_date: Instant::now(),
            timescale: (TimeScale::_1, TimeUnit::Ns),
        }
    }
}

impl Default for VcdScopeModule {
    fn default() -> Self {
        VcdScopeModule {
            name: "TOP".to_string(),
            child_scopes: BTreeMap::new(),
            child_vars: BTreeMap::new(),
        }
    }
}

impl VcdScopeModule {
    pub fn new(name: &ScopeName) -> Self {
        VcdScopeModule {
            name: name.clone(),
            child_scopes: BTreeMap::new(),
            child_vars: BTreeMap::new(),
        }
    }

    pub fn add_var(&mut self, name: &str, ident: &str, vtype: VarType, vsize: u32) -> &mut VcdVar {
        if !self.child_vars.contains_key(name) {
            self.child_vars
                .insert(name.to_owned(), VcdVar::new(name, ident, vtype, vsize));
        }
        self.child_vars.get_mut(name).unwrap()
    }

    pub fn add_scope(&mut self, name: ScopeName) -> &mut VcdScopeModule {
        if !self.child_scopes.contains_key(&name) {
            self.child_scopes
                .insert(name.clone(), VcdScopeModule::new(&name));
        }
        self.child_scopes.get_mut(&name).unwrap()
    }

    pub fn get_var(&self, name: &str) -> Option<&VcdVar> {
        self.child_vars.get(name)
    }

    pub fn get_scope(&self, name: &str) -> Option<&VcdScopeModule> {
        self.child_scopes.get(name)
    }
}

impl Default for VcdVar {
    fn default() -> Self {
        VcdVar {
            vtype: VarType::Wire,
            vsize: 1,
            ident: "!".to_string(),
            reference: "value".to_string(),
        }
    }
}

impl Default for VcdFile {
    fn default() -> Self {
        VcdFile {
            header: VcdHeaderMeta::default(),
            root_scope: VcdScopeModule::default(),
        }
    }
}
