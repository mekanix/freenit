use topcoat::{
    Result,
    context::Cx,
    router::query_params,
    view::{View, component, view},
};

const PER_PAGE: usize = 10;

pub struct Pagination {
    pub page: usize,
    pub pages: usize,
    start: usize,
}

impl Pagination {
    pub fn new(cx: &Cx, total: usize) -> Result<Self> {
        let query = query_params::<PageQuery>(cx)?;
        let pages = total.div_ceil(PER_PAGE).max(1);
        let page = query.page.unwrap_or(1).clamp(1, pages);
        Ok(Self {
            page,
            pages,
            start: (page - 1) * PER_PAGE,
        })
    }

    pub fn slice<T>(&self, values: Vec<T>) -> Vec<T> {
        values
            .into_iter()
            .skip(self.start)
            .take(PER_PAGE)
            .collect()
    }
}

#[query_params(error = bad_request)]
struct PageQuery {
    page: Option<usize>,
}

#[component]
pub async fn pagination(cx: &Cx, base: &str, page: usize, pages: usize) -> Result<impl View> {
    Ok(view! {
        <div class="management-pagination">
            if page > 1 {
                topcoat::runtime::link(
                    href: format!("{base}?page={}", page - 1),
                    attrs: topcoat::view::attributes! { class="button" },
                    "<"
                )
            } else {
                <button class="button" type="button" disabled="">"<"</button>
            }
            (page)
            if page < pages {
                topcoat::runtime::link(
                    href: format!("{base}?page={}", page + 1),
                    attrs: topcoat::view::attributes! { class="button" },
                    ">"
                )
            } else {
                <button class="button" type="button" disabled="">">"</button>
            }
        </div>
    })
}
