use yew::prelude::*;

use crate::timeline::Timeline;

pub struct FavViewer {
    //boot_articles: Option<Vec<ArticleData>>
}

pub enum FavViewerMsg {
    //FetchedBootArticles(Vec<serde_json::Value>),
}

impl Component for FavViewer {
    type Message = FavViewerMsg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            //boot_articles: None,
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
			<div id="timelineContainer">
				<Timeline name="FavViewer"/>
			</div>
		}
    }
}