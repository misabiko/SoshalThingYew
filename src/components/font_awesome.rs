use yew::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum IconType {
	Solid,
	Regular,
	Brand,
}

impl IconType {
	pub fn class(&self) -> &'static str {
		match self {
			IconType::Solid => "fas",
			IconType::Regular => "far",
			IconType::Brand => "fab",
		}
	}
}

#[derive(Clone, Copy, PartialEq)]
pub enum IconSize {
	ExtraSmall,
	Small,
	Large,
	X2,
	X3,
	X5,
	X7,
	X10,
}

impl IconSize {
	pub fn class(&self) -> &'static str {
		match self {
			IconSize::ExtraSmall => "fa-xs",
			IconSize::Small => "fa-sm",
			IconSize::Large => "fa-lg",
			IconSize::X2 => "fa-2x",
			IconSize::X3 => "fa-3x",
			IconSize::X5 => "fa-5x",
			IconSize::X7 => "fa-7x",
			IconSize::X10 => "fa-10x",
		}
	}
}

pub struct FA;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
	#[prop_or(IconType::Solid)]
	pub icon_type: IconType,
	pub icon: String,
	#[prop_or_default]
	pub icon_classes: Classes,
	#[prop_or_default]
	pub span_classes: Classes,
	#[prop_or_default]
	pub size: Option<IconSize>,
}

impl Component for FA {
	type Message = ();
	type Properties = Props;

	fn create(_ctx: &Context<Self>) -> Self {
		Self
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		html! {
			<span class={classes!("icon", ctx.props().span_classes.clone())}>
				<i class={classes!(format!("{} fa-{} {}",
						ctx.props().icon_type.class(),
						ctx.props().icon,
						ctx.props().size.map(|s| s.class()).unwrap_or(""),
					), ctx.props().icon_classes.clone()
					)}
				/>
			</span>
		}
	}
}