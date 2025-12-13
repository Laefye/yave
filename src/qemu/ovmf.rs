use crate::qemu::QEMU;

impl QEMU {
    pub fn ovmf<C: AsRef<std::path::Path>, V: AsRef<std::path::Path>>(mut self, code: C, vars: V) -> Self {
        self.args.push("-drive".to_string());
        self.args.push(format!(
            "if=pflash,format=raw,readonly=on,file={}",
            code.as_ref().to_string_lossy()
        ));
        self.args.push("-drive".to_string());
        self.args.push(format!(
            "if=pflash,format=raw,file={}",
            vars.as_ref().to_string_lossy()
        ));
        self
    }
}