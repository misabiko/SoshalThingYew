use std::cell::RefCell;
use yew::prelude::*;
use std::collections::HashSet;
use std::rc::Rc;

use super::Props;
use crate::articles::{ArticleComponent, ArticleBox};
use crate::timeline::ArticleStruct;

pub struct MasonryContainer {
	cached_articles: Vec<ArticleStruct>,
	columns: Vec<(Vec<ArticleStruct>, f32)>,
}

pub enum Msg {

}

impl Component for MasonryContainer {
	type Message = Msg;
	type Properties = Props;

	fn create(ctx: &Context<Self>) -> Self {
		Self {
			cached_articles: Vec::new(),
			columns: vec![(Vec::<ArticleStruct>::new(), 0.0 as f32); ctx.props().column_count as usize],
			//columns: to_columns(ctx.props().articles.iter(), &ctx.props().column_count, &ctx.props().rtl).collect(),
		}
	}

	fn changed(&mut self, ctx: &Context<Self>) -> bool {
		let mut sorted_cached = self.cached_articles.clone();
		//sorted_cached.sort_by_key(|a| a.global_id());
		let mut sorted_articles = ctx.props().articles.clone();
		//sorted_articles.sort_by_key(|a| a.global_id());

		/*{
			let mut i = 0;
			let mut cached_it = sorted_cached.iter();
			let mut it = sorted_articles.iter();
			let mut str = String::new();
			loop {
				match (it.next(), cached_it.next()) {
					(None, None) => break,
					t => str.push_str(&format!("{} - {} - {}\n", i, t.0.map(|a| a.global_id()).unwrap_or_default(), t.1.map(|a| a.global_id()).unwrap_or_default())),
				};
				i += 1;
			}
			log::debug!("{}", str);
		}
		let mut cached_it = sorted_cached.iter().peekable();
		let mut it = sorted_articles.iter();*/

		let mut added = Vec::new();
		let mut removed = Vec::new();

		for a in &sorted_articles {
			if let Some(index) = sorted_cached.iter().position(|c| c.global_id() == a.global_id()) {
				sorted_cached.remove(index);
			}else {
				added.push(a);
			}
		}

		removed.extend(sorted_cached);

		/*let mut last_cache = None;
		log::debug!("Start loop!");
		loop {
			match (it.next(), last_cache.or_else(|| cached_it.next())) {
				(None, None) => {
					log::debug!("\tbreak!");
					break;
				}
				(Some(article), None) => {
					log::debug!("\tnew article, no more cached!");
					added.push(article);
					last_cache = None;
				}
				(None, Some(cached_article)) => {
					log::debug!("\tremoved article!");
					removed.push(cached_article);
					last_cache = None;
				}
				(Some(article), Some(cached_article)) if &article.global_id() == cached_article => {
					log::debug!("\tsame article");
					last_cache = None;
					continue;
				}
				//new article in props
				(Some(article), Some(cached_article)) => {
					log::debug!("\tnew article!");
					added.push(article);
					last_cache = Some(cached_article);
				}
			};
		}*/

		'outer: for a in removed.into_iter() {
			//log::debug!("Removing {}", a.global_id());
			for (column, mut height) in self.columns.iter_mut() {
				if let Some(index) = column.iter().position(|c_article| c_article.global_id() == a.global_id()) {
					column.remove(index);
					height -= relative_height(&a.boxed);
					continue 'outer;
				}
			}
		}

		for a in added.into_iter() {
			//log::debug!("Adding {}", a.global_id());
			let mut smallest = self.columns.iter_mut().min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap()).unwrap();
			smallest.1 += relative_height(&a.boxed);
			smallest.0.push(a.clone());
		}


		for column in &mut self.columns {
			let mut new_column = Vec::<ArticleStruct>::new();
			for a in &sorted_articles {
				if column.0.iter().find(|c_article| c_article.global_id() == a.global_id()).is_some() {
					new_column.push(a.clone())
				}
			}
			let height = new_column.iter().map(|a| relative_height(&a.boxed)).sum();
			*column = (new_column, height);
		}

		self.cached_articles = ctx.props().articles.clone();
		true
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		let article_view = ctx.props().article_view.clone();
		html! {
			<div class="articlesContainer masonryContainer" ref={ctx.props().container_ref.clone()}>
				{ for self.columns.iter().enumerate().map(|(column_index, (column, _))| html! {
					<div class="masonryColumn" key={column_index}>
						{ for column.iter().enumerate().map(|(load_priority, article_struct)| html! {
							<ArticleComponent
								key={format!("{:?}{}", &article_view, article_struct.boxed.id())}
								article_struct={(*article_struct).clone()}
								{article_view}
								compact={ctx.props().compact}
								animated_as_gifs={ctx.props().animated_as_gifs}
								hide_text={ctx.props().hide_text}
								lazy_loading={ctx.props().lazy_loading}
								load_priority={load_priority as u32 + column_index as u32 * ctx.props().column_count as u32}
								column_count={ctx.props().column_count}
								app_settings={ctx.props().app_settings}
							/>
						}) }
					</div>
				})}
			</div>
		}
	}
}

type RatioedArticle<'a> = (&'a ArticleStruct, f32);
type Column<'a> = (u8, Vec<RatioedArticle<'a>>);

fn relative_height(article: &ArticleBox) -> f32 {
	(1.0 as f32) + article
		.media().iter()
		.map(|m| m.ratio.get())
		.sum::<f32>()
}

fn height(column: &Vec<RatioedArticle>) -> f32 {
	if column.is_empty() {
		0.0
	} else {
		column.iter()
			.map(|r| r.1)
			.sum::<f32>()
	}
}

fn to_columns<'a>(articles: impl Iterator<Item=&'a ArticleStruct>, column_count: &'a u8, rtl: &bool) -> impl Iterator<Item=(impl Iterator<Item=&'a ArticleStruct>, f32)> {
	let ratioed_articles = articles.map(|a| (a, relative_height(&a.boxed)));

	let mut columns = ratioed_articles.fold(
		(0..*column_count)
			.map(|i| (i, Vec::new()))
			.collect::<Vec<Column>>(),
		|mut cols, article| {
			cols.sort_by(|a, b| {
				let h_a = height(&a.1);
				let h_b = height(&b.1);
				h_a.partial_cmp(&h_b).unwrap()//.expect(&format!("comparing {} and {}\n{:#?}\n{:#?}", h_a, h_b, a, b))
			});
			cols[0].1.push(article);
			cols
		},
	);

	columns.sort_by(if *rtl {
		|a: &Column, b: &Column| b.0.partial_cmp(&a.0).unwrap()
	} else {
		|a: &Column, b: &Column| a.0.partial_cmp(&b.0).unwrap()
	});

	//columns.into_iter().map(|c| c.1.into_iter().map(|r| r.0))
	columns.into_iter().map(|c| {
		let height = height(&c.1);
		(c.1.into_iter().map(|r| r.0), height)
	})
}