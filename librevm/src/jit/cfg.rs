use revm::primitives::SpecId;
use revmc::OptimizationLevel;

pub struct JitCfg {
    pub aot: bool,
    pub opt_level: OptimizationLevel,
    pub no_gas: bool,
    pub no_len_checks: bool,
    pub debug_assertions: bool,
    pub eof: bool,
    pub spec_id: SpecId,
}

impl Default for JitCfg {
    fn default() -> Self {
        JitCfg {
            aot: true,
            opt_level: OptimizationLevel::Aggressive,
            no_gas: true,
            no_len_checks: true,
            debug_assertions: true,
            eof: false,
            spec_id: SpecId::PRAGUE,
        }
    }
}
