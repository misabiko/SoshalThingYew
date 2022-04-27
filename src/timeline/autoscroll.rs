use wasm_bindgen::closure::Closure;
use web_sys::{Element, MouseEvent};
use yew::NodeRef;
use std::rc::Rc;
use std::cell::RefCell;
use std::ops::Not;
use wasm_bindgen::JsCast;

pub fn start_autoscroll(autoscroll: &Rc<RefCell<AutoScroll>>, scrolled_ref: NodeRef) {
	/*{
		let mut autoscroll_borrow = autoscroll.borrow_mut();
		if autoscroll_borrow.anim.is_some() {
			log::debug!("invert direction");
			autoscroll_borrow.direction = !autoscroll_borrow.direction;
		}else {
			log::debug!("keep direction");
		}
	}*/

	let anim_autoscroll = autoscroll.clone();
	let event_autoscroll = autoscroll.clone();

	let mut outer_borrow_mut = autoscroll.borrow_mut();

	let window = web_sys::window().expect("no global window");
	outer_borrow_mut.anim = {
		let anim = AutoscrollAnim {
			scroll_step: Closure::wrap(Box::new(move || {
				let mut borrow = anim_autoscroll.borrow_mut();
				if let Some(container) = scrolled_ref.cast::<Element>() {
					let should_keep_scrolling = match borrow.direction {
						ScrollDirection::Up => container.scroll_top() > 0,
						ScrollDirection::Down => container.scroll_top() < container.scroll_height() - container.client_height(),
					};

					if should_keep_scrolling {
						container.scroll_by_with_x_and_y(0.0, match borrow.direction {
							ScrollDirection::Up => -borrow.speed,
							ScrollDirection::Down => borrow.speed,
						});
					} else {
						borrow.direction = !borrow.direction;
					}
				}

				let mut anim = borrow.anim.as_mut().unwrap();
				anim.request_id = web_sys::window().expect("no global window")
					.request_animation_frame(anim.scroll_step.as_ref().unchecked_ref())
					.unwrap();
			}) as Box<dyn FnMut()>),
			request_id: 0,
			scroll_stop: Closure::once(Box::new(move |e: MouseEvent| {
				let mut borrow = event_autoscroll.borrow_mut();
				if let Some(anim) = &borrow.anim {
					web_sys::window().expect("no global window")
						.cancel_animation_frame(anim.request_id)
						.unwrap();
				}

				borrow.anim = None;

				let target = e.target().unwrap();
				let target = target.dyn_ref::<Element>().unwrap();
				//TODO Make sure same timeline
				if target.matches(".timelineAutoscroll, .timelineAutoscroll *").unwrap() {
					borrow.direction = !borrow.direction;
				}
			})),
		};
		let mut options = web_sys::AddEventListenerOptions::new();
		window.add_event_listener_with_callback_and_add_event_listener_options(
			"mousedown",
			anim.scroll_stop.as_ref().unchecked_ref(),
			options.once(true),
		).unwrap();

		window.request_animation_frame(anim.scroll_step.as_ref().unchecked_ref()).unwrap();
		Some(anim)
	};
}

pub fn scroll_to_top(scrolled_element: Element) {
	let mut options = web_sys::ScrollToOptions::new();
	options.top(0.0);
	options.behavior(web_sys::ScrollBehavior::Smooth);
	scrolled_element.scroll_to_with_scroll_to_options(&options);
}

struct AutoscrollAnim {
	request_id: i32,
	scroll_step: Closure<dyn FnMut()>,
	scroll_stop: Closure<dyn FnMut(MouseEvent)>,
}

pub struct AutoScroll {
	direction: ScrollDirection,
	speed: f64,
	anim: Option<AutoscrollAnim>,
}

impl Default for AutoScroll {
	fn default() -> Self {
		Self {
			direction: ScrollDirection::Up,
			speed: 3.0,
			anim: None,
		}
	}
}

#[derive(Clone, Copy)]
enum ScrollDirection {
	Up,
	Down,
}

impl Not for ScrollDirection {
	type Output = ScrollDirection;

	fn not(self) -> Self::Output {
		match self {
			ScrollDirection::Up => ScrollDirection::Down,
			ScrollDirection::Down => ScrollDirection::Up,
		}
	}
}