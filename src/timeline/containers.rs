use yew::prelude::*;
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::cmp::Ordering;

use crate::error::Result;
use crate::articles::{ArticleData, ArticleComponent, ArticleView};

/*Make containers dynamic?
	Would require to dynamically list container names without an enum/vec
	Would require to dynamically create a container from said name
 */
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
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
	pub articles: Vec<(Weak<RefCell<dyn ArticleData>>, Box<dyn ArticleData>)>,
	pub lazy_loading: bool,
}

impl PartialEq for Props {
	fn eq(&self, other: &Self) -> bool {
		self.compact == other.compact &&
			self.animated_as_gifs == other.animated_as_gifs &&
			self.hide_text == other.hide_text &&
			self.column_count == other.column_count &&
			self.lazy_loading == other.lazy_loading &&
			self.article_view == other.article_view &&
			self.articles.len() == other.articles.len() &&
			self.articles.iter().zip(other.articles.iter())
				.all(|((weak_a, a), (weak_b, b))| Weak::ptr_eq(&weak_a, &weak_b) && a == b)
	}
}

#[function_component(ColumnContainer)]
pub fn column_container(props: &Props) -> Html {
	html! {
		<div class="articlesContainer columnContainer" ref={props.container_ref.clone()}>
			{ for props.articles.iter().enumerate().map(|(load_priority, (weak_ref, article))| html! {
				<ArticleComponent
					key={article.id()}
					weak_ref={weak_ref.clone()}
					article={article.clone_data()}
					article_view={props.article_view.clone()}
					compact={props.compact}
					animated_as_gifs={props.animated_as_gifs}
					hide_text={props.hide_text}
					lazy_loading={props.lazy_loading}
					load_priority={load_priority as u32}
					column_count={props.column_count}
				/>
			}) }
		</div>
	}
}

#[function_component(RowContainer)]
pub fn row_container(props: &Props) -> Html {
	let style = match props.rtl {
		true => Some("direction: rtl"),
		false => None,
	};
	html! {
		<div class="articlesContainer rowContainer" ref={props.container_ref.clone()} {style}>
			{ for props.articles.iter().enumerate().map(|(load_priority, (weak_ref, article))| { html! {
				<ArticleComponent
					key={article.id()}
					weak_ref={weak_ref.clone()}
					article={article.clone_data()}
					article_view={props.article_view.clone()}
					compact={props.compact}
					animated_as_gifs={props.animated_as_gifs}
					hide_text={props.hide_text}
					lazy_loading={props.lazy_loading}
					style={format!("width: calc(100% / {})", props.column_count)}
					load_priority={load_priority as u32}
					column_count={props.column_count}
				/>
			}}) }
		</div>
	}
}

type ArticleTuple = (Rc<RefCell<dyn ArticleData>>, Box<dyn ArticleData>);
type RatioedArticle<'a> = (&'a ArticleTuple, f32);
type Column<'a> = (u8, Vec<RatioedArticle<'a>>);

fn relative_height(article: &Box<dyn ArticleData>) -> f32 {
	(1.0 as f32) + article
		.media().iter()
		.map(|m| m.ratio)
		.sum::<f32>()
}

fn height(column: &Column) -> f32 {
	if column.1.is_empty() {
		0.0
	}else {
		column.1.iter()
			.map(|r| r.1)
			.sum::<f32>()
	}
}

fn to_columns<'a>(articles: impl Iterator<Item = &'a ArticleTuple>, column_count: &'a u8, rtl: &bool) -> impl Iterator<Item = impl Iterator<Item = &'a ArticleTuple>> {
	let ratioed_articles = articles.map(|t| (t, relative_height(&t.1)));

	let mut columns = ratioed_articles.fold(
		(0..*column_count)
			.map(|i| (i, Vec::new()))
			.collect::<Vec::<Column>>(),
		|mut cols, article| {
			cols.sort_by(|a, b| height(a).partial_cmp(&height(b)).unwrap_or(Ordering::Equal));
			cols[0].1.push(article);
			cols
		}
	);

	columns.sort_by(if *rtl {
		|a: &Column, b: &Column| b.0.partial_cmp(&a.0).unwrap()
	}else {
		|a: &Column, b: &Column| a.0.partial_cmp(&b.0).unwrap()
	});

	columns.into_iter().map(|c| c.1.into_iter().map(|r| r.0))
}

#[function_component(MasonryContainer)]
pub fn masonry_container(props: &Props) -> Html {
	let strongs: Vec<ArticleTuple> = props.articles.iter().filter_map(|t| t.0.upgrade().map(|s| (s, t.1.clone_data()))).collect();
	let columns = to_columns(strongs.iter(), &props.column_count, &props.rtl);

	html! {
		<div class="articlesContainer masonryContainer" ref={props.container_ref.clone()}>
			{ for columns.enumerate().map(|(column_index, column)| html! {
				<div class="masonryColumn" key={column_index}>
					{ for column.enumerate().map(|(load_priority, (strong_ref, article))| html! {
						<ArticleComponent
							key={article.id()}
							weak_ref={Rc::downgrade(strong_ref)}
							article={article.clone_data()}
							article_view={props.article_view.clone()}
							compact={props.compact}
							animated_as_gifs={props.animated_as_gifs}
							hide_text={props.hide_text}
							lazy_loading={props.lazy_loading}
							load_priority={load_priority as u32 + column_index as u32 * props.column_count as u32}
							column_count={props.column_count}
						/>
					}) }
				</div>
			})}
		</div>
	}
}