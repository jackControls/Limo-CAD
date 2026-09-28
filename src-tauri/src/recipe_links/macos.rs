//! Receive GetURL Apple events without replacing Winit's application delegate.
//! The bundle's CFBundleURLTypes owns registration; callbacks only queue source.
use crate::native_viewport::interface_shell::NativeInterfaceHandle;
use bevy::prelude::*;
use objc2::{
    define_class, msg_send, rc::Retained, sel, DefinedClass, MainThreadMarker, MainThreadOnly,
};
use objc2_foundation::{NSAppleEventDescriptor, NSAppleEventManager, NSObject, NSObjectProtocol};
use std::{
    cell::RefCell,
    collections::VecDeque,
    sync::{Arc, Mutex},
};

#[derive(Resource, Clone, Default)]
struct Pending(Arc<Mutex<VecDeque<String>>>);
struct Delivery {
    pending: Pending,
    wake: NativeInterfaceHandle,
}
define_class!(
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    #[ivars = Delivery]
    struct Receiver;
    unsafe impl NSObjectProtocol for Receiver {}
    impl Receiver {
        #[unsafe(method(receiveRecipe:withReply:))]
        fn receive(&self,event:&NSAppleEventDescriptor,_reply:&NSAppleEventDescriptor) {
            let Some(uri)=event.paramDescriptorForKeyword(u32::from_be_bytes(*b"----")).and_then(|value|value.stringValue()) else {return};
            let recipe=match nbcad_mcp::recipe_id_from_uri(&uri.to_string()) {
                Ok(recipe)=>recipe,
                Err(error)=>{eprintln!("Recipe link rejected: {error}");return;}
            };
            if let Ok(mut queue)=self.ivars().pending.0.lock() {
                if queue.iter().any(|queued|queued==recipe) {return;}
                if queue.len()>=16 {eprintln!("Recipe link queue is full; finish opening pending recipes");return;}
                queue.push_back(recipe.into());
            }
            self.ivars().wake.request_redraw();
        }
    }
);
thread_local! { static RECEIVER:RefCell<Option<Retained<Receiver>>>=const {RefCell::new(None)}; }

pub(super) fn install(app: &mut App) {
    // Winit's event loop has already initialized AppKit, before its first run.
    let mtm = MainThreadMarker::new().expect("Native recipe URLs install on the AppKit thread");
    let pending = Pending::default();
    let wake = app.world().resource::<NativeInterfaceHandle>().clone();
    let receiver: Retained<Receiver> = unsafe {
        msg_send![
            Receiver::alloc(mtm).set_ivars(Delivery {
                pending: pending.clone(),
                wake
            }),
            init
        ]
    };
    unsafe {
        NSAppleEventManager::sharedAppleEventManager()
            .setEventHandler_andSelector_forEventClass_andEventID(
                &receiver,
                sel!(receiveRecipe:withReply:),
                u32::from_be_bytes(*b"GURL"),
                u32::from_be_bytes(*b"GURL"),
            );
    }
    RECEIVER.with(|slot| *slot.borrow_mut() = Some(receiver));
    app.insert_resource(pending).add_systems(Update, deliver);
}
fn deliver(world: &mut World) {
    let pending = world.resource::<Pending>().clone();
    let requests = pending
        .0
        .lock()
        .map(|mut queue| queue.drain(..).collect::<Vec<_>>())
        .unwrap_or_default();
    for recipe in requests {
        crate::session_bridge::native_interface::controller::open_startup_recipe(world, &recipe);
    }
}
