use anymap::AnyMap;
use dioxus::prelude::*;
use dioxus_core::SchedulerMsg;
use dioxus_native_core::real_dom::RealDom;
use renderer::{run, SkiaDom, EventEmitter};
use state::node::NodeState;
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc};
use elements_namespace as dioxus_elements;

pub use renderer;

pub struct LaunchParams {
    pub devtools: bool
}

fn create_win(root: Component<()>, trig_render: Sender<()>) -> (SkiaDom, EventEmitter){
    let rdom = Arc::new(Mutex::new(RealDom::<NodeState>::new()));
    let event_emitter: Arc<Mutex<Option<UnboundedSender<SchedulerMsg>>>> =
        Arc::new(Mutex::new(None));

    {
        let rdom = rdom.clone();
        let event_emitter = event_emitter.clone();
        std::thread::spawn(move || {
            let mut dom = VirtualDom::new(root);

            let muts = dom.rebuild();
            let to_update = rdom.lock().unwrap().apply_mutations(vec![muts]);
            let mut ctx = AnyMap::new();
            ctx.insert(0.0f32);
            rdom.lock().unwrap().update_state(&dom, to_update, ctx);

            event_emitter
                .lock()
                .unwrap()
                .replace(dom.get_scheduler_channel());

            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async move {
                    loop {
                        dom.wait_for_work().await;
                        let mutations = dom.work_with_deadline(|| false);

                        let to_update = rdom.lock().unwrap().apply_mutations(mutations);
                        let ctx = AnyMap::new();
                        if !to_update.is_empty() {
                            trig_render.send(()).unwrap();
                        }
                        rdom.lock().unwrap().update_state(&dom, to_update, ctx);
                    }
                });
        });
    }

    (rdom, event_emitter)
}

pub fn launch(app: Component<()>, params: Option<LaunchParams>) {

    let (trig_render, rev_render) = mpsc::channel::<()>();
    

    let mut windows = vec![
        create_win(app, trig_render.clone())
    ];

    if let Some(params) = params {
        if params.devtools {
            windows.push(create_win(devtools_app, trig_render))
        }
    }

    run(windows, rev_render);
}

fn devtools_app(cx: Scope) -> Element {

    cx.render(rsx!(
        view {
            background: "green",
            width: "100%",
            height: "100%",
            text {
                "hi"
            }
        }
    ))
}
