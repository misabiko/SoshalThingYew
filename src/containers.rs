use yew::prelude::*;
use std::rc::Rc;

use crate::articles::{SocialArticle, SocialArticleData};

//TODO Make containers dynamic
pub enum Container {
	Column,
	Row,
	Masonry,
}

impl Container {
	pub fn name(&self) -> &'static str {
		match self {
			Container::Column => "Column",
			Container::Row => "Row",
			Container::Masonry => "Masonry",
		}
	}
}

pub fn view_container(container: &Container, props: Props) -> Html {
	match container {
		Container::Column => html! {
			<ColumnContainer ..props/>
		},
		Container::Row => html! {
			<RowContainer ..props/>
		},
		Container::Masonry => html! {
			<MasonryContainer ..props/>
		}
	}
}

#[derive(Properties)]
pub struct Props {
	pub compact: bool,
	#[prop_or(1)]
	pub column_count: u8,
	pub articles: Vec<Rc<dyn SocialArticleData>>
}

impl PartialEq for Props {
	fn eq(&self, other: &Self) -> bool {
		self.compact == other.compact &&
		self.column_count == other.column_count &&
			self.articles.len() == other.articles.len() &&
			!self.articles.iter().zip(other.articles.iter())
				.any(|(ai, bi)| !Rc::ptr_eq(&ai, &bi))
	}
}

/*struct ColumnContainer;

impl Component for ColumnContainer {
	type Message = ();
	type Properties = Props;

	fn create(_ctx: &Context<Self>) -> Self {
		Self {}
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		html! {
			<div class="articlesContainer columnContainer">
				{ for ctx.props().articles.iter().map(|data| html! {
					<SocialArticle compact={ctx.props().compact} data={data.clone()}/>
				})}
			</div>
		}
	}
}*/


#[function_component(ColumnContainer)]
pub fn column_container(props: &Props) -> Html {
	html! {
		<div class="articlesContainer columnContainer">
			{ for props.articles.iter().map(|data| html! {
				<SocialArticle compact={props.compact} data={data.clone()}/>
			})}
		</div>
	}
}

//TODO Support rtl
#[function_component(RowContainer)]
pub fn row_container(props: &Props) -> Html {
	html! {
		<div class="articlesContainer rowContainer">
			{ for props.articles.iter().map(|data| html! {
				<SocialArticle
					compact={props.compact}
					data={data.clone()}
					style={format!("width: {}%", 100.0 / (props.column_count as f64))}
				/>
			})}
		</div>
	}
}

#[function_component(MasonryContainer)]
pub fn masonry_container(props: &Props) -> Html {
	html! {
		<div class="articlesContainer masonryContainer">
			{ for props.articles.iter().map(|data| html! {
				<SocialArticle
					compact={props.compact}
					data={data.clone()}
					style={format!("width: {}%", 100.0 / (props.column_count as f64))}
				/>
			})}
		</div>
	}
}