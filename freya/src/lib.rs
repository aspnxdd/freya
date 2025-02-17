use anymap::AnyMap;
use dioxus::prelude::*;
use dioxus_core::SchedulerMsg;
use dioxus_native_core::real_dom::RealDom;
use freya_node_state::node::NodeState;
use freya_renderer::run;
use std::sync::Arc;
use std::sync::Mutex;

pub use freya_components::*;
pub use freya_elements as dioxus_elements;
pub use freya_hooks::*;
pub use freya_renderer::*;

pub fn launch(app: Component<()>) {
    let rdom = Arc::new(Mutex::new(RealDom::<NodeState>::new()));
    let event_emitter: Arc<Mutex<Option<UnboundedSender<SchedulerMsg>>>> =
        Arc::new(Mutex::new(None));

    {
        let rdom = rdom.clone();
        let event_emitter = event_emitter.clone();
        std::thread::spawn(move || {
            let mut dom = VirtualDom::new(app);

            let muts = dom.rebuild();
            let to_update = rdom.lock().unwrap().apply_mutations(vec![muts]);
            let ctx = AnyMap::new();
            rdom.lock().unwrap().update_state(&dom, to_update, ctx);

            event_emitter
                .lock()
                .unwrap()
                .replace(dom.get_scheduler_channel());

            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async move {
                    loop {
                        dom.wait_for_work().await;
                        let mutations = dom.work_with_deadline(|| false);
                        let to_update = rdom.lock().unwrap().apply_mutations(mutations);
                        let ctx = AnyMap::new();
                        rdom.lock().unwrap().update_state(&dom, to_update, ctx);
                    }
                });
        });
    }

    run(rdom, event_emitter);
}
