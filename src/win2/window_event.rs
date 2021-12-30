use std::collections::HashMap;
use std::sync::atomic::{AtomicIsize, Ordering, AtomicBool};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crossbeam_channel::{unbounded, Receiver, Sender};
use lazy_static::lazy_static;

use bindings::Windows::Win32::Foundation::HWND;
use bindings::Windows::Win32::UI::Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent};
use bindings::Windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW,
    PM_REMOVE,
    EVENT_MAX, EVENT_MIN, EVENT_OBJECT_CLOAKED, EVENT_OBJECT_DESTROY, EVENT_OBJECT_FOCUS, EVENT_OBJECT_HIDE, EVENT_OBJECT_SHOW, EVENT_OBJECT_UNCLOAKED, EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_MINIMIZEEND, EVENT_SYSTEM_MINIMIZESTART, EVENT_SYSTEM_MOVESIZEEND, EVENT_SYSTEM_MOVESIZESTART,
    MSG, PeekMessageW, TranslateMessage, EVENT_OBJECT_LOCATIONCHANGE, EVENT_OBJECT_CREATE};

use crate::win2::message_loop::MessageLoop;

use super::error::Result;
use super::window::Window;

lazy_static! {
    static ref WINDOWS_EVENT_CHANNEL: Arc<Mutex<(Sender<WinEvent>, Receiver<WinEvent>)>> =
        Arc::new(Mutex::new(unbounded()));

    static ref EVENTS_CHANNELS: Arc<Mutex<HashMap<isize, Arc<Mutex<(Sender<WinEvent>, Receiver<WinEvent>)>>>>> = 
        Arc::new(Mutex::new(HashMap::new()));
}

pub trait EventHandler {
    fn handle(&mut self, evt: &WinEvent);
}

// implement EventHandler for pure function
impl <F>EventHandler for F
where
    F: FnMut(&WinEvent)
{
    fn handle(&mut self, evt: &WinEvent) {
        self(evt)
    }
}

// unsafe impl <F>Sync for F
// where
//     F: FnMut(&WinEvent)
// {
    
// }

// add ext for window
impl Window {

    // // create listener from window handle
    // pub fn listen(hwnd: HWND) -> WinEventListener {
    //     WinEventListener::new(Window::from(hwnd).unwrap_or_default())
    // }

    pub fn listen(&self) -> WinEventListener {
        // return event listener
        WinEventListener::new(*self)
    }

}

pub struct WinEventListener {
    w: Window,

    hook: AtomicIsize, // sotre the handle id
    exited: Arc<AtomicBool>, // exit the thead


    ch: Arc<Mutex<(Sender<WinEvent>, Receiver<WinEvent>)>>,

    // filter functions: all should be true
    // filters: Arc<Mutex<Vec<Box<dyn FnMut(&WinEvent) -> bool + Send>>>>,
    // handle functions,
    handlers: Arc<Mutex<HashMap<WinEventType, Vec<Box<dyn EventHandler + Send + Sync + 'static>>>>>,
    // handlers: Arc<Mutex<HashMap<WinEventType, Box<dyn EventHandler + Send + Sync + 'static>>>>,

    thread: Option<JoinHandle<()>>, // thread for handle message
}

// pub struct ListenerWrapper(Arc<Mutex<WinEventListener>>);

impl WinEventListener {

    pub fn new(w: Window) -> Self {
        WinEventListener{
            w,

            hook: AtomicIsize::new(0),
            exited: Arc::new(AtomicBool::new(false)),

            ch: Arc::new(Mutex::new(unbounded())),

            // filters: Arc::new(Mutex::new(Vec::<_>::new())),
            handlers: Arc::new(Mutex::new(HashMap::new())),

            thread: None,
        }
    }

    // on method to add event listener, evt type -> callback
    pub fn on<Q>(&mut self, typ: WinEventType, cb: Q) -> &mut Self
    where
        Q: EventHandler + Send + Sync + 'static
    {
        // TODO: add event listener by config
        self.handlers.lock().unwrap().entry(typ)
            .or_insert_with(Vec::new)
            .push(Box::new(cb));

        self
    }

    pub fn start(&mut self, block: bool) -> Result<()> {

        // install the win event hook function
        let hook_handle = unsafe {
            SetWinEventHook(
                EVENT_MIN, 
                EVENT_MAX, 
                None, 
                Some(thunk), 
                0, 
                0, 
                0,
            )
        };

        // take the ch with hook_id?
        println!("the event hook id {:?}", hook_handle);

        self.hook.store(hook_handle.0, Ordering::SeqCst);

        // start the message loop thread
        // and set the event hook handle for w32
        // set to global static send
        EVENTS_CHANNELS.lock().unwrap().insert(hook_handle.0, self.ch.clone());

        let ch = self.ch.clone();
        let _handlers = self.handlers.clone();
        let _exited = self.exited.clone();
        // let _filters = self.filters.clone();
        let target_w = self.w;

        let process = move || {
            if let Ok(evt) = ch.lock().unwrap().1.try_recv() {
                // filter and call with event type
                // for f in _filters.lock().unwrap().into_iter() {
                //     if !f(&evt) {
                //         // if with false just ingore
                //         return true;
                //     }
                // }

                // hard code for window match
                if target_w.is_valide() && evt.window != target_w {
                    // return true;
                    return;
                }

                // call functions with type
                match _handlers.lock().unwrap().get_mut(&evt.etype) {
                    Some(v) => {
                        for cb in v.into_iter() {
                            cb.handle(&evt);
                        }
                    },
                    None => {},
                }

                // call functions all
                match _handlers.lock().unwrap().get_mut(&WinEventType::All) {
                    Some(v) => {
                        for cb in v.into_iter() {
                            cb.handle(&evt);
                        }
                    },
                    None => {},
                }

            }

        };

        if block {
            // start the message loop
            MessageLoop::start(10, |_msg| {
                process();

                true
            });
        } else {
            // store the thread handle
            self.thread = Some(thread::spawn(move || {
                MessageLoop::start(10, |_msg| {
                    process();
    
                    true
                });
            }));
        }

        Ok(())
    }
}

impl Drop for WinEventListener {
    fn drop(&mut self) {
        // unhook the window
        let hid = self.hook.load(Ordering::SeqCst);
        unsafe {
            UnhookWinEvent(HWINEVENTHOOK(hid));
        }
        println!("remove the hook {}", hid);

        // exit thread
        self.exited.store(true, Ordering::SeqCst);
    }
}

// global hook send event to static global queue
// global single thread process the event
// send to each single tread queue

// handle the window hook event process
#[allow(non_snake_case)]
unsafe extern "system" fn thunk(
    hook_handle: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    _id_object: i32,
    _id_child: i32,
    _id_event_thread: u32,
    _dwms_event_time: u32,
) {
    // create the event, add more id fields
    let mut evt = WinEvent::new(hook_handle, event, hwnd);
    evt.raw_id_child = _id_child;
    evt.raw_id_object = _id_object;
    evt.raw_id_thread = _id_event_thread;

    // TODO: add filter at here ingore windows not match???

    // TODO: send to queue

    if evt.etype == WinEventType::Unknown {
        // println!("unknown event type");
        return;
    }

    // geet the handle channel
    match EVENTS_CHANNELS.lock().unwrap().get(&hook_handle.0) {
        None => {
            println!("can't get event channel {:?}", evt.etype);
        },
        Some(v) => {
            v.lock().unwrap().0
            .send(evt)
            .expect("could not send message event channel");
        }
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
pub enum WinEventType {
    Destroy,
    Create,
    Show,
    Hide,
    FocusChange,
    MoveResizeStart,
    MoveResizeEnd,
    LocationChange,

    Unknown,
    All,
}

impl From<u32> for WinEventType {

    fn from(event: u32) -> Self {
        // new evt type from event id
        match event {
            EVENT_OBJECT_DESTROY => Self::Destroy,
            EVENT_OBJECT_CREATE => Self::Create,

            EVENT_OBJECT_CLOAKED
            | EVENT_OBJECT_HIDE
            | EVENT_SYSTEM_MINIMIZESTART => Self::Hide,

            EVENT_OBJECT_SHOW
            | EVENT_OBJECT_UNCLOAKED
            | EVENT_SYSTEM_MINIMIZEEND => Self::Show,

            EVENT_OBJECT_FOCUS
            | EVENT_SYSTEM_FOREGROUND => Self::FocusChange,

            EVENT_SYSTEM_MOVESIZESTART => Self::MoveResizeStart,
            EVENT_SYSTEM_MOVESIZEEND => Self::MoveResizeEnd,

            EVENT_OBJECT_LOCATIONCHANGE => Self::LocationChange,
            
            _ => Self::Unknown,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct WinEvent {
    pub etype: WinEventType,
    pub window: Window,

    // original data
    pub hook_handle: HWINEVENTHOOK,
    pub raw_event: u32,
    pub raw_id_child: i32,
    pub raw_id_object: i32,
    pub raw_id_thread: u32,
}

impl WinEvent {

    // create the event from args
    pub fn new(hook_handle: HWINEVENTHOOK, event: u32, hwnd: HWND) -> Self {
        Self{
            etype: event.into(),
            window: hwnd.into(),

            hook_handle,
            raw_event: event,
            raw_id_child: 0,
            raw_id_object: 0,
            raw_id_thread: 0,
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::win2::{window::*, window_event::{WinEventType, WinEvent}, message_loop::MessageLoop};

    #[test]
    fn test_init_hook() {
        assert_eq!(1, 1);

        let child = Window::from_name(None, "MINGW64:/d/Zoe").unwrap();

        // let _ = Window::default() // for all windows
        let mut listener = Window::from_name(None, "MINGW64:/c/Users/Zoe").unwrap().listen();
        let _ = listener
            .on(WinEventType::MoveResizeStart, |evt: &WinEvent| {
                println!("===> object move start {}!", evt.window);
            })
            .on(WinEventType::LocationChange, |evt: &WinEvent| {
                println!("===> object location change {}!", evt.window);
            })
            .on(WinEventType::MoveResizeEnd, move |evt: &WinEvent| {
                // get the position and set to the child one
                if let Ok(rect) = evt.window.rect() {
                    child.set_pos(rect.right_top())
                }
            }).start(false);

        let mut lis = Window::default().listen();
        // if evt.window.class() == "WeChatMainWndForPC" {
        let _ = lis
            .on(WinEventType::Create, move |evt: &WinEvent| {
                if evt.window.is_valide() {
                    if let Some(title) = evt.window.title() {
                        if title == "微信" {
                            println!("object create {} {} {}", evt.window, evt.raw_id_child, evt.raw_id_object);
                        }
                    }
                }
            })
            .start(false);

        // 是否能够将代码放到静态的hook函数中去

        // 不同线程有问题!!!
        MessageLoop::start(10, |_| { true })
    }
}