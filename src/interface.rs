use futures_util::TryStreamExt;
use rtnetlink::{Handle, LinkUnspec, new_connection, packet_route::link::LinkMessage};

async fn get_interface(handle: &Handle, interface: &str) -> Result<Option<LinkMessage>, rtnetlink::Error> {
    let mut links = handle.link().get().match_name(interface.to_string()).execute();

    links.try_next().await
}

pub async fn set_master(interface: &str, master: &str) -> Result<(), rtnetlink::Error> {
    let (connection, handle, _) = new_connection().unwrap();
    tokio::spawn(connection);
    
    let interface_id = match get_interface(&handle, interface).await? {
        Some(interface) => interface.header.index,
        None => return Ok(()),
    };

    let master_id = match get_interface(&handle, master).await? {
        Some(interface) => interface.header.index,
        None => return Ok(()),
    };

    handle.link().set(LinkUnspec::new_with_index(interface_id).controller(master_id).build()).execute().await?;
    Ok(())
}

pub async fn set_link_up(interface: &str) -> Result<(), rtnetlink::Error> {
    let (connection, handle, _) = new_connection().unwrap();
    tokio::spawn(connection);

    let interface_id = match get_interface(&handle, interface).await? {
        Some(interface) => interface.header.index,
        None => return Ok(()),
    };
    
    handle.link().set(LinkUnspec::new_with_index(interface_id).up().build()).execute().await?;

    Ok(())
}
