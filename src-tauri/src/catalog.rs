use super::roblox::get_json;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

pub async fn categories() -> Result<Value, String> {
    get_json("https://catalog.roblox.com/v1/categories", None).await
}

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: u64,

    pub item_type: String,
    pub asset_type: Option<u64>,
    pub name: String,
    pub description: String,
    pub creator: String,
    pub creator_verified: bool,

    pub price: Option<u64>,
    pub lowest_resale: Option<u64>,
    pub favorites: u64,
    pub limited: bool,
    pub thumb: Option<String>,
}

pub fn item(v: &Value) -> Option<Item> {
    let restrictions: Vec<&str> = v["itemRestrictions"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|r| r.as_str())
        .collect();
    let off_sale =
        v["priceStatus"] == "Off Sale" || (v["price"].is_null() && v["lowestPrice"].is_null());
    Some(Item {
        id: v["id"].as_u64()?,
        item_type: v["itemType"].as_str()?.to_string(),
        asset_type: v["assetType"].as_u64(),
        name: v["name"].as_str().unwrap_or_default().to_string(),
        description: v["description"].as_str().unwrap_or_default().to_string(),
        creator: v["creatorName"].as_str().unwrap_or_default().to_string(),
        creator_verified: v["creatorHasVerifiedBadge"].as_bool().unwrap_or(false),
        price: if off_sale {
            None
        } else {
            v["lowestPrice"].as_u64().or(v["price"].as_u64())
        },
        lowest_resale: v["lowestResalePrice"].as_u64().filter(|p| *p > 0),
        favorites: v["favoriteCount"].as_u64().unwrap_or(0),
        limited: restrictions
            .iter()
            .any(|r| r.contains("Limited") || *r == "Collectible"),
        thumb: None,
    })
}

pub async fn thumbs(items: &[(String, u64)]) -> HashMap<(String, u64), String> {
    let mut out = HashMap::new();
    for kind in ["Asset", "Bundle"] {
        let ids: Vec<u64> = items
            .iter()
            .filter(|(k, _)| k == kind)
            .map(|(_, id)| *id)
            .collect();
        for chunk in ids.chunks(100) {
            let list = chunk
                .iter()
                .map(u64::to_string)
                .collect::<Vec<_>>()
                .join(",");
            let url = if kind == "Asset" {
                format!(
                    "https://thumbnails.roblox.com/v1/assets?assetIds={list}&returnPolicy=PlaceHolder&size=420x420&format=Png&isCircular=false"
                )
            } else {
                format!(
                    "https://thumbnails.roblox.com/v1/bundles/thumbnails?bundleIds={list}&size=420x420&format=Png&isCircular=false"
                )
            };
            if let Ok(v) = get_json(&url, None).await {
                for d in v["data"].as_array().into_iter().flatten() {
                    if let (Some(id), Some(u)) = (d["targetId"].as_u64(), d["imageUrl"].as_str()) {
                        out.insert((kind.to_string(), id), u.to_string());
                    }
                }
            }
        }
    }
    out
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    pub items: Vec<Item>,
    pub next_cursor: Option<String>,
}

pub async fn search(
    keyword: &str,
    category: u64,
    subcategory: Option<u64>,
    sort: u8,
    cursor: Option<String>,
) -> Result<Page, String> {
    if sort > 5 {
        return Err("Unknown sort order.".into());
    }
    let mut url = format!(
        "https://catalog.roblox.com/v2/search/items/details?Category={category}&SortType={sort}&Limit=30"
    );
    if let Some(s) = subcategory {
        url += &format!("&Subcategory={s}");
    }
    if sort == 1 || sort == 2 {
        url += "&SortAggregation=5";
    }
    let k = keyword.trim();
    if !k.is_empty() {
        url += &format!(
            "&Keyword={}",
            urlencoding::encode(&k.chars().take(50).collect::<String>())
        );
    }
    if let Some(c) = cursor.filter(|c| !c.is_empty()) {
        url += &format!("&Cursor={}", urlencoding::encode(&c));
    }
    let v = get_json(&url, None).await.map_err(|e| {
        if e.contains("429") {
            "Roblox is limiting catalog searches. Wait a few seconds and try again.".into()
        } else {
            e
        }
    })?;
    let mut items: Vec<Item> = v["data"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(item)
        .collect();
    let t = thumbs(
        &items
            .iter()
            .map(|i| (i.item_type.clone(), i.id))
            .collect::<Vec<_>>(),
    )
    .await;
    for i in &mut items {
        i.thumb = t.get(&(i.item_type.clone(), i.id)).cloned();
    }
    Ok(Page {
        items,
        next_cursor: v["nextPageCursor"].as_str().map(str::to_string),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_items() {
        let for_sale = item(&json!({ "id": 5, "itemType": "Asset", "assetType": 8, "name": "Hat", "price": 95, "lowestPrice": 95, "creatorName": "Roblox", "favoriteCount": 6, "itemRestrictions": [] })).unwrap();
        assert_eq!(
            (for_sale.price, for_sale.limited, for_sale.asset_type),
            (Some(95), false, Some(8))
        );
        let off = item(&json!({ "id": 6, "itemType": "Asset", "priceStatus": "Off Sale", "itemRestrictions": ["Limited"] })).unwrap();
        assert_eq!((off.price, off.limited), (None, true));
        let free =
            item(&json!({ "id": 7, "itemType": "Bundle", "price": 0, "lowestPrice": 0 })).unwrap();
        assert_eq!(free.price, Some(0));
        assert!(item(&json!({ "name": "no id" })).is_none());
    }
}
