/// this file will be the integration tests base for techniques compilation to cfengine file
/// calling files from the *techniques* folder
/// takes an rl file and a cf file, parses the first, compiles, and compares expected output with the second
/// TODO: cf files added (soloely in case of success) for later use: comparing it to generated output (.rl.cf)
/// naming convention: 
/// input is rl -> state_checkdef.rl
/// output is .rl.cf -> state_checkdef.rl.cf
/// with success: s_ & failure: f_
/// example of files that should succeed: s_errors.rl s_errors.rl.cf errors.cf

#[cfg(test)]
pub fn compile() {}