use crate::{
    config::NodesConfig,
    metric::{observor::MetricObservor, publisher::MetricPublisher},
    // module_iter::*,
    // module_view::setup_views,
    network::p2p::P2PModule,
};
use crate::{
    dummy_fs::{
        cache_router::CacheRouter, fs_node::FsNode, s3_source::S3Source, sim_user::SimUser,
    },
    // kv::{data_router::DataRouter, data_router_client::DataRouterClient, kv_client::KVClient},
    // module_iter::LogicalModuleParent,
    // network::p2p::P2PModule,
    result::WSResult,
    util::JoinHandleWrapper,
};
use async_trait::async_trait;
use lazy_static::lazy_static;
use std::{
    ops::Add,
    sync::{Arc, Weak},
};
use tokio::sync::Mutex;

pub struct Sys {
    pub logical_modules: Arc<LogicalModules>,
    sub_tasks: Mutex<Vec<JoinHandleWrapper>>,
}

impl Sys {
    pub fn new(config: NodesConfig) -> Sys {
        Sys {
            logical_modules: LogicalModules::new(config),
            sub_tasks: Vec::new().into(),
        }
    }
    pub async fn wait_for_end(&mut self) {
        if let Err(err) = self.logical_modules.start(self).await {
            panic!("start logical nodes error: {:?}", err);
        }
        tracing::info!("modules all started, waiting for end");
        for task in self.sub_tasks.lock().await.iter_mut() {
            task.join().await;
        }
    }
}

pub type NodeID = u32;

#[derive(Clone)]
pub struct LogicalModuleNewArgs {
    pub logical_modules_ref: LogicalModulesRef,
    pub parent_name: String,
    pub btx: BroadcastSender,
    pub logical_models: Option<Weak<LogicalModules>>,
    pub nodes_config: NodesConfig,
}

impl LogicalModuleNewArgs {
    pub fn expand_parent_name(&mut self, this_name: &str) {
        let name = format!("{}::{}", self.parent_name, this_name);
        self.parent_name = name;
    }
}

#[async_trait]
pub trait LogicalModule: Send + Sync + 'static {
    fn inner_new(args: LogicalModuleNewArgs) -> Self
    where
        Self: Sized;
    async fn start(&self) -> WSResult<Vec<JoinHandleWrapper>>;
    // async fn listen_async_signal(&self) -> tokio::sync::broadcast::Receiver<LogicalModuleState>;
    // fn listen_sync_signal(&self) -> tokio::sync::broadcast::Receiver<LogicalModuleState>;
}

#[derive(Clone, Debug)]
pub enum BroadcastMsg {
    SysEnd,
}

pub type BroadcastSender = tokio::sync::broadcast::Sender<BroadcastMsg>;

// #[derive(LogicalModuleParent)]

// 使用trait的目的是为了接口干净
// #[derive(ModuleView)]

macro_rules! logical_modules_ref_impl {
    ($module:ident,$type:ty) => {
        impl LogicalModulesRef {
            pub fn $module(&self) -> &$type {
                unsafe {
                    (*self.inner.as_ref().unwrap().as_ptr())
                        .$module
                        .as_ref()
                        .unwrap()
                }
            }
        }
    };
}

macro_rules! logical_modules_refs {
    ($module:ident,$t:ty) => {
        logical_modules_ref_impl!($module,$t);
    };
    ($module:ident,$t:ty,$($modules:ident,$ts:ty),+) => {
        // logical_modules_ref_impl!($module,$t);
        logical_modules_refs!($module,$t);
        logical_modules_refs!($($modules,$ts),+);
    };
}

macro_rules! count_modules {
    ($module:ident,$t:ty) => {1usize};
    ($module:ident,$t:ty,$($modules:ident,$ts:ty),+) => {1usize + count_modules!($($modules,$ts),+)};
}

macro_rules! logical_modules {
    // outter struct
    ($($modules:ident,$ts:ty),+)=>{
        #[derive(Default)]
        pub struct LogicalModules {
            new_cnt:usize,
            start_cnt:usize,
            $(pub $modules: Option<$ts>),+
        }

        logical_modules_refs!($($modules,$ts),+);
        lazy_static! {
            /// This is an example for using doc comment attributes
            static ref ALL_MODULES_COUNT: usize = count_modules!($($modules,$ts),+);
        }
        // $(impl $modules(&self)->&$ts{
        //     self.$modules.as_ref().unwrap()
        // })*
    }
}

#[derive(Clone)]
pub struct LogicalModulesRef {
    inner: Option<Weak<LogicalModules>>,
}

impl LogicalModulesRef {
    pub fn new(inner: Arc<LogicalModules>) -> LogicalModulesRef {
        let inner = Arc::downgrade(&inner);
        LogicalModulesRef { inner: Some(inner) }
    }
}
// impl LogicalModulesRef {
//     fn setup(&mut self, modules: Arc<LogicalModules>) {
//         self.inner = Some(Arc::downgrade(&modules));
//     }
// }

macro_rules! logical_module_view_impl {
    ($module:ident,$module_name:ident,Option<$type:ty>) => {
        impl $module {
            pub fn $module_name(&self) -> &$type {
                self.inner.$module_name().as_ref().unwrap()
            }
        }
    };
    ($module:ident,$module_name:ident,$type:ty) => {
        impl $module {
            pub fn $module_name(&self) -> &$type {
                self.inner.$module_name()
            }
        }
    };
    ($module:ident) => {
        #[derive(Clone)]
        pub struct $module {
            inner: LogicalModulesRef,
        }
        impl $module {
            pub fn new(inner: LogicalModulesRef) -> Self {
                $module { inner }
            }
            // fn setup(&mut self, modules: Arc<LogicalModules>) {
            //     self.inner.setup(modules);
            // }
        }
    };
}

macro_rules! start_module_opt {
    ($self:ident,$sys:ident,$opt:ident) => {
        unsafe {
            let mu = ($self as *const LogicalModules) as *mut LogicalModules;
            (*mu).start_cnt += 1;
        }
        // let _ = STARTED_MODULES_COUNT.fetch_add(1, Ordering::SeqCst);
        if let Some($opt) = $self.$opt.as_ref().unwrap() {
            $sys.sub_tasks.lock().await.append(&mut $opt.start().await?);
        }
    };
}

macro_rules! start_module {
    ($self:ident,$sys:ident,$opt:ident) => {
        // let _ = STARTED_MODULES_COUNT.fetch_add(1, Ordering::SeqCst);
        unsafe {
            let mu = ($self as *const LogicalModules) as *mut LogicalModules;
            (*mu).start_cnt += 1;
        }
        $sys.sub_tasks
            .lock()
            .await
            .append(&mut $self.$opt.as_ref().unwrap().start().await?);
    };
}

logical_modules!(
    p2p,
    P2PModule,
    metric_publisher,
    MetricPublisher,
    metric_observor,
    Option<MetricObservor>,
    s3_source,
    Option<S3Source>,
    fs_node,
    Option<FsNode>,
    cache_router,
    Option<CacheRouter>,
    sim_user,
    Option<SimUser>
);

logical_module_view_impl!(P2PView);
logical_module_view_impl!(P2PView, p2p, P2PModule);

logical_module_view_impl!(MetricObservorView);
logical_module_view_impl!(MetricObservorView, p2p, P2PModule);
logical_module_view_impl!(MetricObservorView, metric_observor, Option<MetricObservor>);

logical_module_view_impl!(MetricPublisherView);
logical_module_view_impl!(MetricPublisherView, p2p, P2PModule);
logical_module_view_impl!(MetricPublisherView, metric_publisher, MetricPublisher);
logical_module_view_impl!(MetricPublisherView, metric_observor, Option<MetricObservor>);

logical_module_view_impl!(S3SourceView);
logical_module_view_impl!(S3SourceView, p2p, P2PModule);
logical_module_view_impl!(S3SourceView, s3_source, Option<S3Source>);

logical_module_view_impl!(FsNodeView);
logical_module_view_impl!(FsNodeView, p2p, P2PModule);
logical_module_view_impl!(FsNodeView, fs_node, Option<FsNode>);
logical_module_view_impl!(FsNodeView, cache_router, Option<CacheRouter>);
logical_module_view_impl!(FsNodeView, metric_publisher, MetricPublisher);

logical_module_view_impl!(CacheRouterView);
logical_module_view_impl!(CacheRouterView, p2p, P2PModule);
logical_module_view_impl!(CacheRouterView, cache_router, Option<CacheRouter>);

logical_module_view_impl!(SimUserView);
logical_module_view_impl!(SimUserView, p2p, P2PModule);
logical_module_view_impl!(SimUserView, fs_node, Option<FsNode>);
logical_module_view_impl!(SimUserView, sim_user, Option<SimUser>);

fn modules_mut_ref(modules: &Arc<LogicalModules>) -> &mut LogicalModules {
    // let _ = SETTED_MODULES_COUNT.fetch_add(1, Ordering::SeqCst);
    let mu = unsafe { &mut *(Arc::downgrade(modules).as_ptr() as *mut LogicalModules) };
    mu.new_cnt += 1;
    mu
}

impl LogicalModules {
    // pub fn iter<'a(&'a self) -> LogicalModuleIter<'a> {
    //     LogicalModuleIter {
    //         logical_modules: self,
    //         index: 0,
    //     }
    // }

    pub fn new(config: NodesConfig) -> Arc<LogicalModules> {
        let (broadcast_tx, _broadcast_rx) = tokio::sync::broadcast::channel::<BroadcastMsg>(1);
        let arc = Arc::new(LogicalModules::default());
        let args = LogicalModuleNewArgs {
            btx: broadcast_tx,
            logical_models: None,
            parent_name: "".to_owned(),
            nodes_config: config.clone(),
            logical_modules_ref: LogicalModulesRef {
                inner: Arc::downgrade(&arc).into(),
            },
        };

        modules_mut_ref(&arc).p2p = Some(P2PModule::new(args.clone()));
        modules_mut_ref(&arc).metric_publisher = Some(MetricPublisher::new(args.clone()));
        modules_mut_ref(&arc).metric_observor = if config.this.1.is_router() {
            Some(Some(MetricObservor::new(args.clone())))
        } else {
            Some(None)
        };
        modules_mut_ref(&arc).s3_source = if config.this.1.is_s3() {
            Some(Some(S3Source::new(args.clone())))
        } else {
            Some(None)
        };
        modules_mut_ref(&arc).fs_node = if config.this.1.is_fs() {
            Some(Some(FsNode::new(args.clone())))
        } else {
            Some(None)
        };
        modules_mut_ref(&arc).cache_router = if config.this.1.is_router() {
            Some(Some(CacheRouter::new(args.clone())))
        } else {
            Some(None)
        };
        modules_mut_ref(&arc).sim_user = if config.this.1.is_fs() {
            Some(Some(SimUser::new(args.clone())))
        } else {
            Some(None)
        };
        // modules_mut_ref(&arc).p2p_kernel = Some(Box::new(P2PQuicNode::new(args.clone())));
        // setup_views(&arc);
        arc
    }

    pub async fn start(&self, sys: &Sys) -> WSResult<()> {
        start_module!(self, sys, p2p);
        start_module!(self, sys, metric_publisher);
        start_module_opt!(self, sys, metric_observor);
        start_module_opt!(self, sys, s3_source);
        start_module_opt!(self, sys, fs_node);
        start_module_opt!(self, sys, cache_router);
        start_module_opt!(self, sys, sim_user);

        assert!(self.start_cnt == ALL_MODULES_COUNT.add(0));
        assert!(self.start_cnt == self.new_cnt);
        // start_module!(self, sys, p2p_kernel);

        // let all_modules_count = ALL_MODULES_COUNT.add(0);
        // assert_eq!(
        //     all_modules_count,
        //     SETTED_MODULES_COUNT.load(Ordering::SeqCst),
        // );

        // assert_eq!(
        //     all_modules_count,
        //     STARTED_MODULES_COUNT.load(Ordering::SeqCst)
        // );

        // tracing::info!(
        //     "all:{}, setted:{}, started:{}",
        //     all_modules_count,
        //     SETTED_MODULES_COUNT.load(Ordering::SeqCst),
        //     STARTED_MODULES_COUNT.load(Ordering::SeqCst)
        // );

        Ok(())
    }
}
