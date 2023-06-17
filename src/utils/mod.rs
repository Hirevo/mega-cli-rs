pub mod nodes;
pub mod terminal;

/// Returns whether the argument is a public MEGA URL rather than a path or a node handle.
pub fn is_public_link(input: &str) -> bool {
    // TODO: Give more thought about what checks should be done here.
    //       MEGAcmd itself turns out to be a bit clueless/handwavy in this departement as well:
    //       https://github.com/meganz/MEGAcmd/blob/179af369c83bd8887c309c35191d4151ed524273/src/megacmdcommonutils.cpp#L146-L169
    input.starts_with("https://")
}
