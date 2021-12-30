use crate::win2::{window::Window, rect::Rect, window_event::{WinEventType, WinEvent}, error::Result};
use std::cmp;

/*
 *                                            
 *                 │                          │
 *                 │                          │
 *                 │ top_left        top_right│
 *                 │                          │
 *                 ├───►                  ◄───┤
 *   ────────────┬─┼──────────────────────────┼─┬────────
 *      left_top │ │                          │ │
 *               ▼ │                          │ │  right_top
 *                 │                          │ ▼
 *                 │                          │
 *                 │                          │
 *                 │                          │ ▲
 *               ▲ │                          │ │
 *               │ │                          │ │
 * left_botttom  │ │                          │ │  right_bottom
 *    ───────────┴─┼──────────────────────────┼─┴────────
 *                 ├───►                ◄─────┤
 *                 │                          │
 *                 │ botom_left   bottom_right│
 *                 │                          │
 *                 │                          │
 *
 */

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AttachDirection {
    LeftTop, TopLeft,
    RightTop, TopRight,
    RightBottom, BottomRight,
    LeftBottom, BottomLeft,
}

impl AttachDirection {

    // we know which direction, so only one max and min
    // just for one direction.
    pub fn match_size(self, current: (i32, i32), target: (i32, i32), min: i32, max: i32) -> (i32, i32) {
        let min_max =|v: i32, min: i32, max: i32| -> i32 {
            let mut m = v;
            if min > 0 {
                m = cmp::max(min, m);
            }
            if max > 0 {
                m = cmp::min(max, m);
            }
            m
        };

        match self {
            Self::TopLeft | Self::TopRight | Self::BottomRight | Self::BottomLeft => {
                (min_max(target.0, min, max), current.1)
            },
            Self::LeftTop | Self::LeftBottom | Self::RightTop | Self::RightBottom => {
                (current.0, min_max(target.1, min, max))
            },
        }
    }

    pub fn apply(self, current: Rect, target: Rect, fixed: (i32, i32)) -> (i32, i32) {
        match self {
            Self::LeftTop => {
                let p = target.left_top();
                (p.0 + fixed.0 - current.width, p.1 + fixed.1)
            },
            Self::TopLeft => {
                let p = target.left_top();
                (p.0 + fixed.0, p.1 + fixed.1 - current.height)
            },
            Self::RightTop => {
                let p = target.right_top();
                (p.0 + fixed.0, p.1 + fixed.1)
            },
            Self::TopRight => {
                let p = target.right_top();
                (p.0 + fixed.0 - current.width, p.1 + fixed.1 - current.height)
            },
            Self::RightBottom => {
                let p = target.right_bottom();
                (p.0 + fixed.0, p.1 + fixed.1 - current.height)

            },
            Self::BottomRight => {
                let p = target.right_bottom();
                (p.0 + fixed.0 - current.width, p.1 + fixed.1)

            },
            Self::LeftBottom => {
                let p = target.left_bottom();
                (p.0 + fixed.0 - current.width, p.1 + fixed.1 - current.height)

            },
            Self::BottomLeft => {
                let p = target.left_bottom();
                (p.0 + fixed.0, p.1 + fixed.1)

            },
        }
    }
}

impl Window {
    // create attach builder
    fn attach_to(&self, target: Window) -> WindowAttach {
        let wa = WindowAttach::new(*self, target);
        wa
    }
}

pub struct WindowAttach {
    // self window
    w: Window,
    // target window
    target: Window,

    // direction
    dir: AttachDirection,
    // match the size or not with max and min limit
    match_size: bool,
    match_size_max: u32,
    match_size_min: u32,
    // fix the position from target
    fix_pos: (i32, i32),
}

impl WindowAttach {

    // constructor for window attach
    pub fn new(w: Window, target: Window) -> Self {
        Self {
            w, target,
            dir: AttachDirection::RightTop,
            match_size: false,
            match_size_max: 0,
            match_size_min: 0,
            fix_pos: (0, 0),
        }
    }

    pub fn dir(&mut self, direction: AttachDirection) -> &mut Self {
        self.dir = direction;
        self
    }

    pub fn match_size(&mut self, enable: bool) -> &mut Self {
        self.match_size = enable;
        self
    }

    pub fn match_size_limit(&mut self, max: u32, min: u32) -> &mut Self {
        self.match_size_max = max;
        self.match_size_min = min;
        self
    }

    pub fn match_size_max(&mut self, max: u32) -> &mut Self {
        self.match_size_max = max;
        self
    }

    pub fn match_size_min(&mut self, min: u32) -> &mut Self {
        self.match_size_min = min;
        self
    }

    pub fn fix_pos(&mut self, fixed: (i32, i32)) -> &mut Self {
        // x, y
        self.fix_pos = fixed;
        self
    }

    // start the attach
    pub fn start(&mut self) -> Result<()> {
        // set the target to be owner
        self.w.set_owner(self.target);


        let _dir = self.dir;
        let _match_size = self.match_size;
        let _max = self.match_size_max;
        let _min = self.match_size_min;
        let _fix_pos = self.fix_pos;
        let _target = self.target;
        let _window = self.w;


        let update_rect = move || {
            // get the rect of target
            let target_rect = _target.rect().unwrap();

            let mut current_rect =  _window.rect().unwrap();
            let old = current_rect;
            // resize self, this must be first!
            // postion needs size
            if _match_size {
                let size = _dir.match_size(
                    (current_rect.width, current_rect.height), 
                    (target_rect.width, target_rect.height),
                    _min as _,
                    _max as _,
                );
                current_rect.width = size.0;
                current_rect.height = size.1;
            }
            // this shoudl be in a function
            let p = _dir.apply(current_rect, target_rect, _fix_pos);
            current_rect.x = p.0;
            current_rect.y = p.1;
            if !old.eq(&current_rect) {
                // update 
                println!("change rect {}", current_rect);
                _window.set_rect(&current_rect, false);
            }
            println!("same one");
        };

        // init udpate
        update_rect();

        // start the event hook
        let mut listener = self.target.listen();
        let _ = listener
            .on(WinEventType::LocationChange, move |evt: &WinEvent| {
                // TODO: too many events
                println!("evt.obejct {}, evt.child {}", evt.raw_id_object, evt.raw_id_child);
                if 0 == evt.raw_id_object { update_rect(); }
            })
            .on(WinEventType::MoveResizeEnd, move |evt: &WinEvent| {
                // reset size and pos
                // get the old place???
                update_rect();
            })
            .start(true);

        Ok(())
    }
}

impl Drop for WindowAttach {
    fn drop(&mut self) {
        println!("window attach droped.");
    }
}

#[cfg(test)]
mod tests {
    use crate::win2::window::Window;

    use super::{AttachDirection};


    #[test]
    fn test_demo() {
        let child = Window::from_name(None, "MINGW64:/d/Zoe").unwrap();
        let target = Window::from_name(None, "MINGW64:/c/Users/Zoe").unwrap();

        let _ = child.attach_to(target)
            .match_size(true)
            .dir(AttachDirection::LeftBottom)
            .match_size_min(200)
            .match_size_max(800)
            .fix_pos((0, 0))
            .start();
    }
}