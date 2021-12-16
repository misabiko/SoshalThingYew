use yew::prelude::*;
use std::rc::Rc;

use crate::articles::{view_article, ArticleData, ArticleComponent};

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
	pub article_component: ArticleComponent,
	pub articles: Vec<Rc<dyn ArticleData>>
}

impl PartialEq for Props {
	fn eq(&self, other: &Self) -> bool {
		self.compact == other.compact &&
		self.column_count == other.column_count &&
			self.article_component == other.article_component &&
			self.articles.len() == other.articles.len() &&
			self.articles.iter().zip(other.articles.iter())
				.all(|(ai, bi)| Rc::ptr_eq(&ai, &bi))
	}
}


#[function_component(ColumnContainer)]
pub fn column_container(props: &Props) -> Html {
	html! {
		<div class="articlesContainer columnContainer">
			{ for props.articles.iter().map(|article| view_article(&props.article_component, article.clone()))}
		</div>
	}
}

//TODO Support rtl
//TODO Support compact and style
#[function_component(RowContainer)]
pub fn row_container(props: &Props) -> Html {
	/* html! {
		<SocialArticle
			compact={props.compact}
			data={data.clone()}
			style={format!("width: {}%", 100.0 / (props.column_count as f64))}
		/>
	}*/
	html! {
		<div class="articlesContainer rowContainer">
			{ for props.articles.iter().map(|article| view_article(&props.article_component, article.clone())) }
		</div>
	}
}

type RatioedArticle<'a> = (&'a Rc<dyn ArticleData>, u32);
type Column<'a> = (u8, Vec<RatioedArticle<'a>>);

//TODO Actually estimate article's size
fn relative_height(article: &Rc<dyn ArticleData>) -> u32 {
	1 + article.media().len() as u32
}

fn height(column: &Column) -> u32 {
	if column.1.is_empty() {
		0
	}else {
		column.1.iter()
			.map(|r| r.1)
			.sum::<u32>()
	}
}

fn to_columns<'a>(articles: impl Iterator<Item = &'a Rc<dyn ArticleData>>, column_count: &'a u8) -> impl Iterator<Item = impl Iterator<Item = &'a Rc<dyn ArticleData>>> {
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
	let columns = to_columns(props.articles.iter(), &props.column_count);

	html! {
		<div class="articlesContainer masonryContainer">
			{ for columns.enumerate().map(|(column_index, column)| html! {
				<div class="masonryColumn" key={column_index}>
					{ for column.map(|article| view_article(&props.article_component, article.clone()))}
				</div>
			})}
		</div>
	}
}