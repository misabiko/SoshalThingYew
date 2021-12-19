use yew::prelude::*;
use std::rc::{Rc, Weak};
use std::cell::RefCell;

use crate::articles::{view_article, ArticleData, ArticleComponent, ArticleMedia};

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
	pub container_ref: NodeRef,
	pub compact: bool,
	pub animated_as_gifs: bool,
	pub hide_text: bool,
	#[prop_or(1)]
	pub column_count: u8,
	pub article_component: ArticleComponent,
	pub articles: Vec<Weak<RefCell<dyn ArticleData>>>
}

impl PartialEq for Props {
	fn eq(&self, other: &Self) -> bool {
		self.compact == other.compact &&
		self.animated_as_gifs == other.animated_as_gifs &&
		self.hide_text == other.hide_text &&
		self.column_count == other.column_count &&
			self.article_component == other.article_component &&
			self.articles.len() == other.articles.len() &&
			self.articles.iter().zip(other.articles.iter())
				.all(|(ai, bi)| Weak::ptr_eq(&ai, &bi))
	}
}


#[function_component(ColumnContainer)]
pub fn column_container(props: &Props) -> Html {
	log::debug!("Container {}", props.articles.len());
	html! {
		<div class="articlesContainer columnContainer" ref={props.container_ref.clone()}>
			{ for props.articles.iter().map(|article| view_article(
				&props.article_component,
				props.compact.clone(),
				props.animated_as_gifs.clone(),
				props.hide_text.clone(),
				None,
				article.clone()))}
		</div>
	}
}

//TODO Support rtl
#[function_component(RowContainer)]
pub fn row_container(props: &Props) -> Html {
	html! {
		<div class="articlesContainer rowContainer" ref={props.container_ref.clone()}>
			{ for props.articles.iter().map(|article| view_article(
				&props.article_component,
				props.compact.clone(),
				props.animated_as_gifs.clone(),
				props.hide_text.clone(),
				Some(format!("width: {}%", 100.0 / (props.column_count as f64))),
				article.clone()
			)) }
		</div>
	}
}

type RatioedArticle<'a> = (&'a Rc<RefCell<dyn ArticleData>>, f32);
type Column<'a> = (u8, Vec<RatioedArticle<'a>>);

fn relative_height(article: &Rc<RefCell<dyn ArticleData>>) -> f32 {
	(1.0 as f32) + article.borrow()
		.media().iter()
		.map(|m| match m {
			ArticleMedia::Image(_, ratio) | ArticleMedia::Video(_, ratio) | ArticleMedia::Gif(_, ratio) => ratio
		})
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

fn to_columns<'a>(articles: impl Iterator<Item = &'a Rc<RefCell<dyn ArticleData>>>, column_count: &'a u8) -> impl Iterator<Item = impl Iterator<Item = &'a Rc<RefCell<dyn ArticleData>>>> {
	let ratioed_articles = articles.map(|a| (a, relative_height(&a)));

	let mut columns = ratioed_articles.fold(
		(0..*column_count)
			.map(|i| (i, Vec::new()))
			.collect::<Vec::<Column>>(),
		|mut cols, article| {
			cols.sort_by(|a, b| height(a).partial_cmp(&height(b)).unwrap());
			cols[0].1.push(article);
			cols
		}
	);

	let rtl = false;
	columns.sort_by(if rtl {
		|a: &Column, b: &Column| b.0.partial_cmp(&a.0).unwrap()
	}else {
		|a: &Column, b: &Column| a.0.partial_cmp(&b.0).unwrap()
	});

	columns.into_iter().map(|c| c.1.into_iter().map(|r| r.0))
}

#[function_component(MasonryContainer)]
pub fn masonry_container(props: &Props) -> Html {
	let strongs: Vec<Rc<RefCell<dyn ArticleData>>> = props.articles.iter().filter_map(|a| a.upgrade()).collect();
	let columns = to_columns(strongs.iter(), &props.column_count);

	html! {
		<div class="articlesContainer masonryContainer" ref={props.container_ref.clone()}>
			{ for columns.enumerate().map(|(column_index, column)| html! {
				<div class="masonryColumn" key={column_index}>
					{ for column.map(|article| view_article(
						&props.article_component,
						props.compact.clone(),
						props.animated_as_gifs.clone(),
						props.hide_text.clone(),
						None,
						Rc::downgrade(article)
					))}
				</div>
			})}
		</div>
	}
}