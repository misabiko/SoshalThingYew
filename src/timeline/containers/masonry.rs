use yew::prelude::*;

use super::Props;
use crate::articles::{ArticleComponent, ArticleBox, ArticleRc};
use crate::timeline::ArticleStruct;

#[function_component(MasonryContainer)]
pub fn masonry_container(props: &Props) -> Html {
	let strongs: Vec<StrongArticleStruct> =
		props.articles.iter()
		.filter_map(|article_struct|
			article_struct.weak.upgrade()
				.map(|strong| (strong, article_struct))
		).collect();

	let &Props {column_count, rtl, ..} = props;
	let columns = to_columns(strongs.iter(), &column_count, &rtl);

	let article_view = props.article_view.clone();
	html! {
		<div class="articlesContainer masonryContainer" ref={props.container_ref.clone()}>
			{ for columns.enumerate().map(|(column_index, column)| html! {
				<div class="masonryColumn" key={column_index}>
					{ for column.enumerate().map(|(load_priority, (_, article_struct))| html! {
						<ArticleComponent
							key={format!("{:?}{}", &article_view, article_struct.boxed.id())}
							article_struct={(*article_struct).clone()}
							{article_view}
							compact={props.compact}
							animated_as_gifs={props.animated_as_gifs}
							hide_text={props.hide_text}
							lazy_loading={props.lazy_loading}
							load_priority={load_priority as u32 + column_index as u32 * props.column_count as u32}
							column_count={props.column_count}
							app_settings={props.app_settings}
						/>
					}) }
				</div>
			})}
		</div>
	}
}

type StrongArticleStruct<'a> = (ArticleRc, &'a ArticleStruct);

type RatioedArticle<'a> = (&'a StrongArticleStruct<'a>, f32);
type Column<'a> = (u8, Vec<RatioedArticle<'a>>);

fn relative_height(article: &ArticleBox) -> f32 {
	(1.0 as f32) + article
		.media().iter()
		.map(|m| m.ratio.get())
		.sum::<f32>()
}

fn height(column: &Column) -> f32 {
	if column.1.is_empty() {
		0.0
	} else {
		column.1.iter()
			.map(|r| r.1)
			.sum::<f32>()
	}
}

fn to_columns<'a>(articles: impl Iterator<Item=&'a StrongArticleStruct<'a>>, column_count: &'a u8, rtl: &bool) -> impl Iterator<Item=impl Iterator<Item=&'a StrongArticleStruct<'a>>> {
	let ratioed_articles = articles.map(|tuple| (tuple, relative_height(&tuple.1.boxed)));

	let mut columns = ratioed_articles.fold(
		(0..*column_count)
			.map(|i| (i, Vec::new()))
			.collect::<Vec<Column>>(),
		|mut cols, article| {
			cols.sort_by(|a, b| {
				let h_a = height(a);
				let h_b = height(b);
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

	columns.into_iter().map(|c| c.1.into_iter().map(|r| r.0))
}