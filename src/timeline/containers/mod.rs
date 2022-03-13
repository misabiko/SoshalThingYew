mod masonry;

use yew::prelude::*;
use std::rc::Weak;
use serde::{Serialize, Deserialize};

pub use masonry::MasonryContainer;
use crate::error::Result;
use crate::articles::{ArticleComponent, ArticleView};
use crate::settings::AppSettings;
use crate::timeline::ArticleStruct;

/*Make containers dynamic?
	Would require to dynamically list container names without an enum/vec
	Would require to dynamically create a container from said name
 */
#[derive(Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Debug)]
pub enum Container {
	Column,
	Row,
	Masonry,
}

impl Default for Container {
	fn default() -> Self {
		Container::Column
	}
}

impl Container {
	pub fn from(name: &str) -> Result<Self> {
		match name {
			"Column" => Ok(Container::Column),
			"Row" => Ok(Container::Row),
			"Masonry" => Ok(Container::Masonry),
			_ => Err(format!("Couldn't parse container \"{}\".", name).into()),
		}
	}
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
	pub container_ref: NodeRef,
	pub compact: bool,
	pub animated_as_gifs: bool,
	pub hide_text: bool,
	#[prop_or(1)]
	pub column_count: u8,
	pub rtl: bool,
	pub article_view: ArticleView,
	pub articles: Vec<ArticleStruct>,
	pub lazy_loading: bool,
	pub app_settings: AppSettings,
}

impl PartialEq for Props {
	fn eq(&self, other: &Self) -> bool {
		self.compact == other.compact &&
			self.animated_as_gifs == other.animated_as_gifs &&
			self.hide_text == other.hide_text &&
			self.column_count == other.column_count &&
			self.lazy_loading == other.lazy_loading &&
			self.article_view == other.article_view &&
			self.app_settings == other.app_settings &&
			self.articles.len() == other.articles.len() &&
			self.articles.iter().zip(other.articles.iter())
				.all(|(a, b)| Weak::ptr_eq(&a.weak, &b.weak) && &a.boxed == &b.boxed && a.boxed_ref == b.boxed_ref)
	}
}

//TODO Pass ArticleStruct whole to article props
#[function_component(ColumnContainer)]
pub fn column_container(props: &Props) -> Html {
	let article_view = props.article_view.clone();
	html! {
		<div class="articlesContainer columnContainer" ref={props.container_ref.clone()}>
			{ for props.articles.iter().enumerate().map(|(load_priority, article_struct)| html! {
				<ArticleComponent
					key={format!("{:?}{}", &article_view, article_struct.boxed.id())}
					weak_ref={article_struct.weak.clone()}
					article={article_struct.boxed.clone_data()}
					ref_article={article_struct.boxed_ref.clone_data()}
					{article_view}
					compact={props.compact}
					animated_as_gifs={props.animated_as_gifs}
					hide_text={props.hide_text}
					lazy_loading={props.lazy_loading}
					load_priority={load_priority as u32}
					column_count=1
					app_settings={props.app_settings}
					included={article_struct.included}
				/>
			}) }
		</div>
	}
}

#[function_component(RowContainer)]
pub fn row_container(props: &Props) -> Html {
	//TODO Keep scroll bar on the right
	let style = match props.rtl {
		true => Some("direction: rtl"),
		false => None,
	};

	let article_view = props.article_view.clone();
	html! {
		<div class="articlesContainer rowContainer" ref={props.container_ref.clone()} {style}>
			{ for props.articles.iter().enumerate().map(|(load_priority, article_struct)| { html! {
				<ArticleComponent
					key={format!("{:?}{}", &article_view, article_struct.boxed.id())}
					weak_ref={article_struct.weak.clone()}
					article={article_struct.boxed.clone_data()}
					ref_article={article_struct.boxed_ref.clone_data()}
					{article_view}
					compact={props.compact}
					animated_as_gifs={props.animated_as_gifs}
					hide_text={props.hide_text}
					lazy_loading={props.lazy_loading}
					style={format!("width: calc(100% / {})", props.column_count)}
					load_priority={load_priority as u32}
					column_count={props.column_count}
					app_settings={props.app_settings}
					included={article_struct.included}
				/>
			}}) }
		</div>
	}
}