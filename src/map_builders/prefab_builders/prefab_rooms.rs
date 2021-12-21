#[derive(Copy, Clone)]
pub struct PrefabRoom {
    pub template: &'static str,
    pub width: usize,
    pub height: usize,
    pub first_depth: i32,
    pub last_depth: i32,
}

pub const TOTALLY_NOT_A_TRAP: PrefabRoom = PrefabRoom {
    template: TOTALLY_NOT_A_TRAP_MAP,
    width: 5,
    height: 5,
    first_depth: 0,
    last_depth: 100,
};

const TOTALLY_NOT_A_TRAP_MAP: &str = "
     
 ^^^ 
 ^!^ 
 ^^^ 
     
";
