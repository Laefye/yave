use crate::context::YaveContext;

pub struct NetworkManager<'ctx> {
    context: &'ctx YaveContext,
}

impl <'ctx> NetworkManager<'ctx> {
    pub fn new(context: &'ctx YaveContext) -> Self {
        Self { context }
    }
}

impl <'ctx> NetworkManager<'ctx> {
    pub async fn up_interface(&self, ifname: &str) -> Result<(), crate::Error> {
        let vm = self.context.registry().get_vm_by_ifname(ifname).await?
            .ok_or(crate::Error::VMNotFound)?;
        println!("Bringing up interface {} for VM {:?}", ifname, vm);
        crate::interface::set_link_up(ifname).await?;
        Ok(())

    }
}
