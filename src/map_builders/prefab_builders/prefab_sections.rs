#[derive(PartialEq, Copy, Clone)]
pub enum HorizontalPlacement {
    //Left,
    Center,
    Right,
}

#[derive(PartialEq, Copy, Clone)]
pub enum VerticalPlacement {
    Top,
    Center,
    //Bottom,
}

#[derive(PartialEq, Copy, Clone)]
pub struct PrefabSection {
    pub template: &'static str,
    pub width: usize,
    pub height: usize,
    pub placement: (HorizontalPlacement, VerticalPlacement),
}

pub const UNDERGROUND_FORT: PrefabSection = PrefabSection {
    template: RIGHT_FORT,
    width: 15,
    height: 43,
    placement: (HorizontalPlacement::Right, VerticalPlacement::Top),
};

const RIGHT_FORT: &str = "     #
  #######
  #     #
  #     #######
  #  g        #
  #     #######
  #     #
  ### ###
    # #
    # #
    # ##
    ^
    ^
    # ##
    # #
    # #
    # #
    # #
  ### ###
  #     #
  #     #
  #  g  #
  #     #
  #     #
  ### ###
    # #
    # #
    # #
    # ##
    ^
    ^
    # ##
    # #
    # #
    # #
  ### ###
  #     #
  #     #######
  #  g        #
  #     #######
  #     #
  #######
     #
";

pub const ORC_CAMP: PrefabSection = PrefabSection {
    template: ORC_CAMP_TXT,
    width: 12,
    height: 12,
    placement: (HorizontalPlacement::Center, VerticalPlacement::Center),
};

const ORC_CAMP_TXT: &str = "

 ≈≈≈≈o≈≈≈≈≈
 ≈☼      ☼≈
 ≈ g      ≈
 ≈        ≈
 ≈     g  ≈
 o    O   o
 ≈        ≈
 ≈ g      ≈
 ≈     g  ≈
 ≈☼      ☼≈
 ≈≈≈≈o≈≈≈≈≈

";
