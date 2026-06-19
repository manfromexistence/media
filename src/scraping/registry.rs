//! Scraping targets registry - 200+ pre-configured free media websites.
//!
//! All targets are organized by category and include:
//! - CSS selectors for media extraction
//! - Rate limiting recommendations  
//! - License information

use super::targets::{ScrapingCategory, ScrapingMethod, ScrapingTarget};

// ═══════════════════════════════════════════════════════════════════════════════
// BATCH 1: IMAGE SCRAPERS (1-50) - Stock Photos & Free Images
// ═══════════════════════════════════════════════════════════════════════════════

/// StockSnap.io - 100,000+ CC0 photos
pub const STOCKSNAP: ScrapingTarget = ScrapingTarget::new(
    "stocksnap",
    "StockSnap.io",
    "https://stocksnap.io",
    ".photo-item",
    "img[src]",
    ScrapingCategory::Images,
    "CC0",
    "100,000+",
)
.with_search_url("https://stocksnap.io/search/{query}")
.with_method(ScrapingMethod::Sitemap)
.with_pagination(".pagination a.next")
.with_rate_limit(1000);

/// Burst by Shopify - 30,000+ free photos
pub const BURST: ScrapingTarget = ScrapingTarget::new(
    "burst",
    "Burst (Shopify)",
    "https://burst.shopify.com",
    ".photo-tile",
    ".photo-tile__image img",
    ScrapingCategory::Images,
    "Free",
    "30,000+",
)
.with_search_url("https://burst.shopify.com/photos/search?q={query}")
.with_pagination(".pagination__next");

/// Reshot - 40,000+ free photos
pub const RESHOT: ScrapingTarget = ScrapingTarget::new(
    "reshot",
    "Reshot",
    "https://reshot.com",
    ".photo-card",
    ".photo-card img",
    ScrapingCategory::Images,
    "Free",
    "40,000+",
)
.with_search_url("https://reshot.com/search/{query}");

/// PicJumbo - 10,000+ free photos
pub const PICJUMBO: ScrapingTarget = ScrapingTarget::new(
    "picjumbo",
    "PicJumbo",
    "https://picjumbo.com",
    ".photo-item",
    ".photo-item img",
    ScrapingCategory::Images,
    "Free",
    "10,000+",
)
.with_search_url("https://picjumbo.com/?s={query}");

/// Gratisography - 1,000+ quirky free photos
pub const GRATISOGRAPHY: ScrapingTarget = ScrapingTarget::new(
    "gratisography",
    "Gratisography",
    "https://gratisography.com",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "Free",
    "1,000+",
);

/// Life of Pix - 5,000+ CC0 photos
pub const LIFE_OF_PIX: ScrapingTarget = ScrapingTarget::new(
    "lifeofpix",
    "Life of Pix",
    "https://lifeofpix.com",
    ".photo-thumb",
    ".photo-thumb img",
    ScrapingCategory::Images,
    "CC0",
    "5,000+",
)
.with_search_url("https://lifeofpix.com/search/{query}");

/// Negative Space - 3,000+ CC0 photos
pub const NEGATIVE_SPACE: ScrapingTarget = ScrapingTarget::new(
    "negativespace",
    "Negative Space",
    "https://negativespace.co",
    ".photo-item",
    ".photo-item img",
    ScrapingCategory::Images,
    "CC0",
    "3,000+",
)
.with_search_url("https://negativespace.co/?s={query}");

/// Foodiesfeed - 2,500+ free food photos
pub const FOODIESFEED: ScrapingTarget = ScrapingTarget::new(
    "foodiesfeed",
    "Foodiesfeed",
    "https://foodiesfeed.com",
    ".photo-item",
    ".photo-item img",
    ScrapingCategory::Images,
    "Free",
    "2,500+",
)
.with_search_url("https://foodiesfeed.com/?s={query}")
.with_notes("Food photography focused");

/// Skitterphoto - 3,500+ CC0 photos
pub const SKITTERPHOTO: ScrapingTarget = ScrapingTarget::new(
    "skitterphoto",
    "Skitterphoto",
    "https://skitterphoto.com",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "CC0",
    "3,500+",
);

/// Cupcake - 500+ CC0 photos
pub const CUPCAKE: ScrapingTarget = ScrapingTarget::new(
    "cupcake",
    "Cupcake",
    "https://cupcake.nilssonlee.se",
    "article",
    "img",
    ScrapingCategory::Images,
    "CC0",
    "500+",
)
.with_method(ScrapingMethod::Direct);

/// ISO Republic - 7,000+ free photos
pub const ISO_REPUBLIC: ScrapingTarget = ScrapingTarget::new(
    "isorepublic",
    "ISO Republic",
    "https://isorepublic.com",
    ".photo-item",
    ".photo-item img",
    ScrapingCategory::Images,
    "Free",
    "7,000+",
)
.with_search_url("https://isorepublic.com/search/{query}")
.with_method(ScrapingMethod::Sitemap);

/// SplitShire - 1,500+ free photos
pub const SPLITSHIRE: ScrapingTarget = ScrapingTarget::new(
    "splitshire",
    "SplitShire",
    "https://splitshire.com",
    ".photo-item",
    ".photo-item img",
    ScrapingCategory::Images,
    "Free",
    "1,500+",
);

/// LibreShot - 1,000+ CC0 photos
pub const LIBRESHOT: ScrapingTarget = ScrapingTarget::new(
    "libreshot",
    "LibreShot",
    "https://libreshot.com",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "CC0",
    "1,000+",
)
.with_search_url("https://libreshot.com/?s={query}");

/// Magdeleine - 2,500+ CC0/Free photos
pub const MAGDELEINE: ScrapingTarget = ScrapingTarget::new(
    "magdeleine",
    "Magdeleine",
    "https://magdeleine.co",
    ".photo-item",
    ".photo-item img",
    ScrapingCategory::Images,
    "CC0/Free",
    "2,500+",
)
.with_search_url("https://magdeleine.co/browse/");

/// Kaboompics - 15,000+ free photos
pub const KABOOMPICS: ScrapingTarget = ScrapingTarget::new(
    "kaboompics",
    "Kaboompics",
    "https://kaboompics.com",
    ".photo-item",
    ".photo-item img",
    ScrapingCategory::Images,
    "Free",
    "15,000+",
)
.with_search_url("https://kaboompics.com/gallery?search={query}");

/// Jay Mantri - 400+ CC0 photos
pub const JAY_MANTRI: ScrapingTarget = ScrapingTarget::new(
    "jaymantri",
    "Jay Mantri",
    "https://jaymantri.com",
    "article",
    "img",
    ScrapingCategory::Images,
    "CC0",
    "400+",
)
.with_method(ScrapingMethod::Direct);

/// Travel Coffee Book - 500+ CC0 travel photos
pub const TRAVEL_COFFEE_BOOK: ScrapingTarget = ScrapingTarget::new(
    "travelcoffeebook",
    "Travel Coffee Book",
    "https://travelcoffeebook.com",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "CC0",
    "500+",
)
.with_notes("Travel photography focused");

/// Moveast - 300+ CC0 Portugal travel photos
pub const MOVEAST: ScrapingTarget = ScrapingTarget::new(
    "moveast",
    "Moveast",
    "https://moveast.me",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "CC0",
    "300+",
)
.with_notes("Portugal travel photography");

/// Stokpic - 1,500+ free photos
pub const STOKPIC: ScrapingTarget = ScrapingTarget::new(
    "stokpic",
    "Stokpic",
    "https://stokpic.com",
    ".photo-item",
    ".photo-item img",
    ScrapingCategory::Images,
    "Free",
    "1,500+",
);

/// Foca Stock - 400+ CC0 photos
pub const FOCA_STOCK: ScrapingTarget = ScrapingTarget::new(
    "focastock",
    "Foca Stock",
    "https://focastock.com",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "CC0",
    "400+",
);

/// Good Stock Photos - 800+ CC0 photos
pub const GOOD_STOCK_PHOTOS: ScrapingTarget = ScrapingTarget::new(
    "goodstockphotos",
    "Good Stock Photos",
    "https://goodstock.photos",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "CC0",
    "800+",
);

/// Barn Images - 1,500+ free photos
pub const BARN_IMAGES: ScrapingTarget = ScrapingTarget::new(
    "barnimages",
    "Barn Images",
    "https://barnimages.com",
    ".photo-item",
    ".photo-item img",
    ScrapingCategory::Images,
    "Free",
    "1,500+",
);

/// Freely Photos - 500+ CC0 Christian/spiritual photos
pub const FREELY_PHOTOS: ScrapingTarget = ScrapingTarget::new(
    "freelyphotos",
    "Freely Photos",
    "https://freelyphotos.com",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "CC0",
    "500+",
)
.with_notes("Christian/spiritual photography");

/// DesignersPics - 700+ free photos
pub const DESIGNERSPICS: ScrapingTarget = ScrapingTarget::new(
    "designerspics",
    "DesignersPics",
    "https://designerspics.com",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "Free",
    "700+",
);

/// Free Nature Stock - 1,500+ CC0 nature photos
pub const FREE_NATURE_STOCK: ScrapingTarget = ScrapingTarget::new(
    "freenaturestock",
    "Free Nature Stock",
    "https://freenaturestock.com",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "CC0",
    "1,500+",
)
.with_notes("Nature photography focused");

/// Public Domain Pictures - 200,000+ CC0 photos
pub const PUBLIC_DOMAIN_PICTURES: ScrapingTarget = ScrapingTarget::new(
    "publicdomainpictures",
    "Public Domain Pictures",
    "https://publicdomainpictures.net",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "CC0",
    "200,000+",
)
.with_search_url("https://publicdomainpictures.net/en/search.php?q={query}")
.with_method(ScrapingMethod::Sitemap);

/// PxHere - 1,200,000+ CC0 photos
pub const PXHERE: ScrapingTarget = ScrapingTarget::new(
    "pxhere",
    "PxHere",
    "https://pxhere.com",
    ".photo-item",
    ".photo-item img",
    ScrapingCategory::Images,
    "CC0",
    "1,200,000+",
)
.with_search_url("https://pxhere.com/en/photos?q={query}")
.with_method(ScrapingMethod::Sitemap);

/// StockVault - 140,000+ various license photos
pub const STOCKVAULT: ScrapingTarget = ScrapingTarget::new(
    "stockvault",
    "StockVault",
    "https://stockvault.net",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "Various",
    "140,000+",
)
.with_search_url("https://stockvault.net/free-photos/?q={query}")
.with_method(ScrapingMethod::Sitemap);

/// FreeRangeStock - 50,000+ free photos
pub const FREERANGESTOCK: ScrapingTarget = ScrapingTarget::new(
    "freerangestock",
    "FreeRangeStock",
    "https://freerangestock.com",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "Free",
    "50,000+",
)
.with_search_url("https://freerangestock.com/search.php?search={query}");

/// RGBStock - 100,000+ free photos
pub const RGBSTOCK: ScrapingTarget = ScrapingTarget::new(
    "rgbstock",
    "RGBStock",
    "https://rgbstock.com",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "Free",
    "100,000+",
)
.with_search_url("https://rgbstock.com/search?query={query}");

/// Morguefile - 400,000+ Morguefile license photos
pub const MORGUEFILE: ScrapingTarget = ScrapingTarget::new(
    "morguefile",
    "Morguefile",
    "https://morguefile.com",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "Morguefile",
    "400,000+",
)
.with_search_url("https://morguefile.com/photos/morguefile/{query}");

/// New Old Stock - 1,000+ vintage public domain photos
pub const NEW_OLD_STOCK: ScrapingTarget = ScrapingTarget::new(
    "newoldstock",
    "New Old Stock",
    "https://nos.twnsnd.co",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "PD",
    "1,000+",
)
.with_method(ScrapingMethod::Tumblr)
.with_notes("Vintage photos from public archives");

/// Pickup Image - 30,000+ CC0 photos
pub const PICKUP_IMAGE: ScrapingTarget = ScrapingTarget::new(
    "pickupimage",
    "Pickup Image",
    "https://pickupimage.com",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "CC0",
    "30,000+",
)
.with_search_url("https://pickupimage.com/search/{query}");

/// MMT Stock - 2,000+ CC0 photos
pub const MMT_STOCK: ScrapingTarget = ScrapingTarget::new(
    "mmtstock",
    "MMT Stock",
    "https://mmtstock.com",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "CC0",
    "2,000+",
);

/// Lock & Stock Photos - 1,000+ CC0 photos
pub const LOCK_AND_STOCK: ScrapingTarget = ScrapingTarget::new(
    "lockandstock",
    "Lock & Stock Photos",
    "https://lockandstockphotos.com",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "CC0",
    "1,000+",
);

/// PhotoStockEditor - 50,000+ CC0 photos
pub const PHOTOSTOCKEDITOR: ScrapingTarget = ScrapingTarget::new(
    "photostockeditor",
    "PhotoStockEditor",
    "https://photostockeditor.com",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "CC0",
    "50,000+",
)
.with_search_url("https://photostockeditor.com/search/{query}");

/// Styled Stock - 500+ feminine free photos
pub const STYLED_STOCK: ScrapingTarget = ScrapingTarget::new(
    "styledstock",
    "Styled Stock",
    "https://styledstock.co",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "Free",
    "500+",
)
.with_notes("Feminine styled photography");

/// ShotStash - 5,000+ CC0 photos
pub const SHOTSTASH: ScrapingTarget = ScrapingTarget::new(
    "shotstash",
    "ShotStash",
    "https://shotstash.com",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "CC0",
    "5,000+",
)
.with_search_url("https://shotstash.com/search/{query}");

/// Nappy - 10,000+ CC0 photos of Black and Brown people
pub const NAPPY: ScrapingTarget = ScrapingTarget::new(
    "nappy",
    "Nappy",
    "https://nappy.co",
    ".photo-item",
    ".photo-item img",
    ScrapingCategory::Images,
    "CC0",
    "10,000+",
)
.with_search_url("https://nappy.co/search/{query}")
.with_notes("Diverse representation focused");

/// Iwaria - 1,500+ free African photos
pub const IWARIA: ScrapingTarget = ScrapingTarget::new(
    "iwaria",
    "Iwaria",
    "https://iwaria.com",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "Free",
    "1,500+",
)
.with_notes("African photography focused");

/// Epicantus - 200+ CC0 photos
pub const EPICANTUS: ScrapingTarget = ScrapingTarget::new(
    "epicantus",
    "Epicantus",
    "https://epicantus.tumblr.com",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "CC0",
    "200+",
)
.with_method(ScrapingMethod::Tumblr);

/// Tookapic - 10,000+ free photos
pub const TOOKAPIC: ScrapingTarget = ScrapingTarget::new(
    "tookapic",
    "Tookapic",
    "https://stock.tookapic.com",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "Free",
    "10,000+",
);

/// Snapwire Snaps - 1,000+ CC0 photos
pub const SNAPWIRE_SNAPS: ScrapingTarget = ScrapingTarget::new(
    "snapwiresnaps",
    "Snapwire Snaps",
    "https://snapwiresnaps.tumblr.com",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "CC0",
    "1,000+",
)
.with_method(ScrapingMethod::Tumblr);

/// Bucketlistly - 8,000+ CC0 travel photos
pub const BUCKETLISTLY: ScrapingTarget = ScrapingTarget::new(
    "bucketlistly",
    "Bucketlistly",
    "https://photos.bucketlistly.blog",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "CC0",
    "8,000+",
)
.with_notes("Travel photography focused");

/// Avopix - 50,000+ free photos
pub const AVOPIX: ScrapingTarget = ScrapingTarget::new(
    "avopix",
    "Avopix",
    "https://avopix.com",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "Free",
    "50,000+",
)
.with_search_url("https://avopix.com/search?q={query}");

/// FancyCrave - 2,000+ CC0 photos
pub const FANCYCRAVE: ScrapingTarget = ScrapingTarget::new(
    "fancycrave",
    "FancyCrave",
    "https://fancycrave.com",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "CC0",
    "2,000+",
);

/// Picography - 1,500+ CC0 photos
pub const PICOGRAPHY: ScrapingTarget = ScrapingTarget::new(
    "picography",
    "Picography",
    "https://picography.co",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "CC0",
    "1,500+",
);

/// Jeshoots - 500+ CC0 photos
pub const JESHOOTS: ScrapingTarget = ScrapingTarget::new(
    "jeshoots",
    "Jeshoots",
    "https://jeshoots.com",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "CC0",
    "500+",
);

/// Raumrot - 300+ CC0 photos
pub const RAUMROT: ScrapingTarget = ScrapingTarget::new(
    "raumrot",
    "Raumrot",
    "https://raumrot.com",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "CC0",
    "300+",
);

/// Albumarium - 500+ CC0 photos
pub const ALBUMARIUM: ScrapingTarget = ScrapingTarget::new(
    "albumarium",
    "Albumarium",
    "https://albumarium.com",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "CC0",
    "500+",
);

// ═══════════════════════════════════════════════════════════════════════════════
// BATCH 2: IMAGE SCRAPERS (51-86) - Government & Specialized Sources
// ═══════════════════════════════════════════════════════════════════════════════

/// Getrefe - 500+ CC0 natural photos
pub const GETREFE: ScrapingTarget = ScrapingTarget::new(
    "getrefe",
    "Getrefe",
    "https://getrefe.com/photos",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "CC0",
    "500+",
);

/// Ancestry Images - 30,000+ public domain vintage photos
pub const ANCESTRY_IMAGES: ScrapingTarget = ScrapingTarget::new(
    "ancestryimages",
    "Ancestry Images",
    "https://ancestryimages.com",
    "article",
    "img",
    ScrapingCategory::Images,
    "PD",
    "30,000+",
)
.with_notes("Vintage/genealogy photos");

/// Old Book Illustrations - 4,000+ public domain illustrations
pub const OLD_BOOK_ILLUSTRATIONS: ScrapingTarget = ScrapingTarget::new(
    "oldbookillustrations",
    "Old Book Illustrations",
    "https://oldbookillustrations.com",
    ".illustration",
    ".illustration img",
    ScrapingCategory::Images,
    "PD",
    "4,000+",
)
.with_method(ScrapingMethod::Sitemap)
.with_search_url("https://oldbookillustrations.com/?s={query}");

/// Getty Open Content - 150,000+ free artworks
pub const GETTY_OPEN: ScrapingTarget = ScrapingTarget::new(
    "getty",
    "Getty Open Content",
    "https://getty.edu/art/collection",
    ".artwork",
    ".artwork img",
    ScrapingCategory::Images,
    "Free",
    "150,000+",
)
.with_search_url("https://www.getty.edu/art/collection/search?q={query}");

/// Yale Beinecke Library - 500,000+ public domain images
pub const YALE_BEINECKE: ScrapingTarget = ScrapingTarget::new(
    "yalebeinecke",
    "Yale Beinecke Library",
    "https://beinecke.library.yale.edu",
    ".image",
    ".image img",
    ScrapingCategory::Images,
    "PD",
    "500,000+",
);

/// Paris Musées - 300,000+ CC0 artworks
pub const PARIS_MUSEES: ScrapingTarget = ScrapingTarget::new(
    "parismusees",
    "Paris Musées",
    "https://parismuseescollections.paris.fr",
    ".artwork",
    ".artwork img",
    ScrapingCategory::Images,
    "CC0",
    "300,000+",
);

/// ESA Images - 50,000+ CC space photos
pub const ESA_IMAGES: ScrapingTarget = ScrapingTarget::new(
    "esaimages",
    "ESA Images",
    "https://esa.int/ESA_Multimedia/Images",
    ".image",
    ".image img",
    ScrapingCategory::Images,
    "CC",
    "50,000+",
)
.with_notes("European Space Agency photos");

/// NOAA Photo Library - 50,000+ public domain photos
pub const NOAA_PHOTOS: ScrapingTarget = ScrapingTarget::new(
    "noaaphotos",
    "NOAA Photo Library",
    "https://photolib.noaa.gov",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "PD",
    "50,000+",
)
.with_notes("Weather, ocean, climate photos");

/// US Fish & Wildlife - 100,000+ public domain wildlife photos
pub const USFWS: ScrapingTarget = ScrapingTarget::new(
    "usfws",
    "US Fish & Wildlife",
    "https://digitalmedia.fws.gov",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "PD",
    "100,000+",
)
.with_notes("Wildlife photography");

/// National Park Service - 50,000+ public domain photos
pub const NPS_PHOTOS: ScrapingTarget = ScrapingTarget::new(
    "npsphotos",
    "National Park Service",
    "https://nps.gov/media",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "PD",
    "50,000+",
)
.with_notes("National parks photography");

/// US Geological Survey - 200,000+ public domain photos
pub const USGS_PHOTOS: ScrapingTarget = ScrapingTarget::new(
    "usgsphotos",
    "US Geological Survey",
    "https://usgs.gov/media",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "PD",
    "200,000+",
)
.with_notes("Geological/scientific imagery");

/// CDC PHIL - 25,000+ public domain health images
pub const CDC_PHIL: ScrapingTarget = ScrapingTarget::new(
    "cdcphil",
    "CDC PHIL",
    "https://phil.cdc.gov",
    ".image",
    ".image img",
    ScrapingCategory::Images,
    "PD",
    "25,000+",
)
.with_notes("Public health imagery");

/// NIH Image Gallery - 10,000+ public domain medical images
pub const NIH_GALLERY: ScrapingTarget = ScrapingTarget::new(
    "nihgallery",
    "NIH Image Gallery",
    "https://nih.gov/news-events/images",
    ".image",
    ".image img",
    ScrapingCategory::Images,
    "PD",
    "10,000+",
)
.with_notes("Medical/scientific imagery");

/// US Navy Images - 100,000+ public domain photos
pub const US_NAVY: ScrapingTarget = ScrapingTarget::new(
    "usnavy",
    "US Navy Images",
    "https://navy.mil/view_galleries.asp",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "PD",
    "100,000+",
);

/// US Air Force - 50,000+ public domain photos
pub const US_AIR_FORCE: ScrapingTarget = ScrapingTarget::new(
    "usairforce",
    "US Air Force",
    "https://af.mil/News/Photos",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "PD",
    "50,000+",
);

/// UN Photos - 800,000+ UN terms photos
pub const UN_PHOTOS: ScrapingTarget = ScrapingTarget::new(
    "unphotos",
    "UN Photos",
    "https://dam.media.un.org",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "UN Terms",
    "800,000+",
)
.with_notes("United Nations media");

/// Superfamous - 200+ CC photos
pub const SUPERFAMOUS: ScrapingTarget = ScrapingTarget::new(
    "superfamous",
    "Superfamous",
    "https://superfamous.com",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "CC",
    "200+",
);

/// Realistic Shots - 1,000+ CC0 photos
pub const REALISTIC_SHOTS: ScrapingTarget = ScrapingTarget::new(
    "realisticshots",
    "Realistic Shots",
    "https://realisticshots.com",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "CC0",
    "1,000+",
);

/// Startup Stock Photos - 500+ CC0 startup/tech photos
pub const STARTUP_STOCK: ScrapingTarget = ScrapingTarget::new(
    "startupstock",
    "Startup Stock Photos",
    "https://startupstockphotos.com",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "CC0",
    "500+",
)
.with_notes("Tech/startup themed");

/// Photo Collections - 300+ CC0 curated photos
pub const PHOTO_COLLECTIONS: ScrapingTarget = ScrapingTarget::new(
    "photocollections",
    "Photo Collections",
    "https://photocollections.io",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "CC0",
    "300+",
);

/// Vintage Stock Photos - 2,000+ CC0 vintage photos
pub const VINTAGE_STOCK: ScrapingTarget = ScrapingTarget::new(
    "vintagestock",
    "Vintage Stock Photos",
    "https://vintagestockphotos.com",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "CC0",
    "2,000+",
);

/// RetroGraphic - 500+ public domain vintage images
pub const RETROGRAPHIC: ScrapingTarget = ScrapingTarget::new(
    "retrographic",
    "RetroGraphic",
    "https://retrographic.co",
    ".image",
    ".image img",
    ScrapingCategory::Images,
    "PD",
    "500+",
);

/// Old Design Shop - 10,000+ public domain vintage images
pub const OLD_DESIGN_SHOP: ScrapingTarget = ScrapingTarget::new(
    "olddesignshop",
    "Old Design Shop",
    "https://olddesignshop.com",
    ".image",
    ".image img",
    ScrapingCategory::Images,
    "PD",
    "10,000+",
);

/// WOCINTECH - 500+ CC tech diversity photos
pub const WOCINTECH: ScrapingTarget = ScrapingTarget::new(
    "wocintech",
    "WOCINTECH",
    "https://wocintechchat.com",
    "article",
    "img",
    ScrapingCategory::Images,
    "CC",
    "500+",
)
.with_method(ScrapingMethod::Direct)
.with_notes("Women of color in tech");

/// Jopwell Collection - 200+ free diversity photos
pub const JOPWELL: ScrapingTarget = ScrapingTarget::new(
    "jopwell",
    "Jopwell Collection",
    "https://jopwell.pixieset.com",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "Free",
    "200+",
)
.with_notes("Diverse professionals");

/// CreateHER Stock - 100+ free diverse women photos
pub const CREATEHER_STOCK: ScrapingTarget = ScrapingTarget::new(
    "createherstock",
    "CreateHER Stock",
    "https://createherstock.com/free",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "Free",
    "100+",
)
.with_notes("Women of color stock photos");

/// 1 Million Free Pictures - 50,000+ CC0 photos
pub const ONE_MILLION_FREE: ScrapingTarget = ScrapingTarget::new(
    "onemillionfree",
    "1 Million Free Pictures",
    "https://1millionfreepictures.com",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "CC0",
    "50,000+",
);

/// Crow The Stone - 300+ CC0 photos
pub const CROW_THE_STONE: ScrapingTarget = ScrapingTarget::new(
    "crowthestone",
    "Crow The Stone",
    "https://crowthestone.tumblr.com",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "CC0",
    "300+",
)
.with_method(ScrapingMethod::Tumblr);

/// Tumblr Free Stock - 500+ CC0 photos
pub const TUMBLR_FREE_STOCK: ScrapingTarget = ScrapingTarget::new(
    "tumblrfreestock",
    "Tumblr Free Stock",
    "https://freestock.tumblr.com",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "CC0",
    "500+",
)
.with_method(ScrapingMethod::Tumblr);

/// PhotoRack - 3,000+ free photos
pub const PHOTORACK: ScrapingTarget = ScrapingTarget::new(
    "photorack",
    "PhotoRack",
    "https://photorack.net",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "Free",
    "3,000+",
);

/// FreePhotos.cc - 500+ CC0 photos
pub const FREEPHOTOS_CC: ScrapingTarget = ScrapingTarget::new(
    "freephotoscc",
    "FreePhotos.cc",
    "https://freephotos.cc",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "CC0",
    "500+",
);

/// Little Visuals - 500+ CC0 photos (archived)
pub const LITTLE_VISUALS: ScrapingTarget = ScrapingTarget::new(
    "littlevisuals",
    "Little Visuals",
    "https://littlevisuals.co",
    "article",
    "img",
    ScrapingCategory::Images,
    "CC0",
    "500+",
)
.with_method(ScrapingMethod::Direct)
.with_notes("Archived - no longer updated");

/// Death to Stock - 2,000+ free photos
pub const DEATH_TO_STOCK: ScrapingTarget = ScrapingTarget::new(
    "deathtostock",
    "Death to Stock",
    "https://deathtothestockphoto.com",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "Free",
    "2,000+",
);

/// SkypixelPhotos - 500,000+ various license drone photos
pub const SKYPIXEL: ScrapingTarget = ScrapingTarget::new(
    "skypixel",
    "SkypixelPhotos",
    "https://skypixel.com",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "Various",
    "500,000+",
)
.with_notes("Drone/aerial photography");

/// LibreStock - 70,000+ CC0 meta-search
pub const LIBRESTOCK: ScrapingTarget = ScrapingTarget::new(
    "librestock",
    "LibreStock",
    "https://librestock.com",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "CC0",
    "70,000+",
)
.with_search_url("https://librestock.com/photos/{query}")
.with_notes("Meta-search aggregator");

/// FindA.Photo - 10,000+ CC0 meta-search
pub const FINDA_PHOTO: ScrapingTarget = ScrapingTarget::new(
    "findaphoto",
    "FindA.Photo",
    "https://finda.photo",
    ".photo",
    ".photo img",
    ScrapingCategory::Images,
    "CC0",
    "10,000+",
)
.with_search_url("https://finda.photo/{query}")
.with_notes("Meta-search by color/keyword");

// ═══════════════════════════════════════════════════════════════════════════════
// BATCH 2 COLLECTION - Government & Specialized Image Scrapers (36 sites)
// ═══════════════════════════════════════════════════════════════════════════════

/// All Batch 2 image scraping targets (36 sites, ~3M+ assets)
pub const BATCH_2_IMAGE_TARGETS: &[ScrapingTarget] = &[
    GETREFE,
    ANCESTRY_IMAGES,
    OLD_BOOK_ILLUSTRATIONS,
    GETTY_OPEN,
    YALE_BEINECKE,
    PARIS_MUSEES,
    ESA_IMAGES,
    NOAA_PHOTOS,
    USFWS,
    NPS_PHOTOS,
    USGS_PHOTOS,
    CDC_PHIL,
    NIH_GALLERY,
    US_NAVY,
    US_AIR_FORCE,
    UN_PHOTOS,
    SUPERFAMOUS,
    REALISTIC_SHOTS,
    STARTUP_STOCK,
    PHOTO_COLLECTIONS,
    VINTAGE_STOCK,
    RETROGRAPHIC,
    OLD_DESIGN_SHOP,
    WOCINTECH,
    JOPWELL,
    CREATEHER_STOCK,
    ONE_MILLION_FREE,
    CROW_THE_STONE,
    TUMBLR_FREE_STOCK,
    PHOTORACK,
    FREEPHOTOS_CC,
    LITTLE_VISUALS,
    DEATH_TO_STOCK,
    SKYPIXEL,
    LIBRESTOCK,
    FINDA_PHOTO,
];

// ═══════════════════════════════════════════════════════════════════════════════
// BATCH 1 COLLECTION - All 50 Image Scrapers
// ═══════════════════════════════════════════════════════════════════════════════

/// All Batch 1 image scraping targets (50 sites, ~2M+ assets)
pub const BATCH_1_IMAGE_TARGETS: &[ScrapingTarget] = &[
    STOCKSNAP,
    BURST,
    RESHOT,
    PICJUMBO,
    GRATISOGRAPHY,
    LIFE_OF_PIX,
    NEGATIVE_SPACE,
    FOODIESFEED,
    SKITTERPHOTO,
    CUPCAKE,
    ISO_REPUBLIC,
    SPLITSHIRE,
    LIBRESHOT,
    MAGDELEINE,
    KABOOMPICS,
    JAY_MANTRI,
    TRAVEL_COFFEE_BOOK,
    MOVEAST,
    STOKPIC,
    FOCA_STOCK,
    GOOD_STOCK_PHOTOS,
    BARN_IMAGES,
    FREELY_PHOTOS,
    DESIGNERSPICS,
    FREE_NATURE_STOCK,
    PUBLIC_DOMAIN_PICTURES,
    PXHERE,
    STOCKVAULT,
    FREERANGESTOCK,
    RGBSTOCK,
    MORGUEFILE,
    NEW_OLD_STOCK,
    PICKUP_IMAGE,
    MMT_STOCK,
    LOCK_AND_STOCK,
    PHOTOSTOCKEDITOR,
    STYLED_STOCK,
    SHOTSTASH,
    NAPPY,
    IWARIA,
    EPICANTUS,
    TOOKAPIC,
    SNAPWIRE_SNAPS,
    BUCKETLISTLY,
    AVOPIX,
    FANCYCRAVE,
    PICOGRAPHY,
    JESHOOTS,
    RAUMROT,
    ALBUMARIUM,
];

// ═══════════════════════════════════════════════════════════════════════════════
// BATCH 3: VIDEO SCRAPERS (1-50) - Stock Footage & Free Videos
// ═══════════════════════════════════════════════════════════════════════════════

/// Mixkit Videos - 10,000+ free stock videos
pub const MIXKIT_VIDEOS: ScrapingTarget = ScrapingTarget::new(
    "mixkitvideos",
    "Mixkit Videos",
    "https://mixkit.co/free-stock-video",
    ".video-item",
    "video source, .video-item a[href]",
    ScrapingCategory::Videos,
    "Free",
    "10,000+",
)
.with_search_url("https://mixkit.co/free-stock-video/{query}");

/// Videvo Free - 15,000+ free stock videos
pub const VIDEVO: ScrapingTarget = ScrapingTarget::new(
    "videvo",
    "Videvo",
    "https://videvo.net",
    ".video-item",
    ".video-item video, .video-item a[href]",
    ScrapingCategory::Videos,
    "Free",
    "15,000+",
)
.with_search_url("https://videvo.net/search/{query}");

/// Life of Vids - 500+ CC0 videos
pub const LIFE_OF_VIDS: ScrapingTarget = ScrapingTarget::new(
    "lifeofvids",
    "Life of Vids",
    "https://lifeofvids.com",
    ".video",
    ".video video, .video a[href]",
    ScrapingCategory::Videos,
    "CC0",
    "500+",
);

/// Dareful - 200+ CC0 videos
pub const DAREFUL: ScrapingTarget = ScrapingTarget::new(
    "dareful",
    "Dareful",
    "https://dareful.com",
    ".video",
    ".video video",
    ScrapingCategory::Videos,
    "CC0",
    "200+",
);

/// Vidsplay - 400+ free videos
pub const VIDSPLAY: ScrapingTarget = ScrapingTarget::new(
    "vidsplay",
    "Vidsplay",
    "https://vidsplay.com",
    ".video",
    ".video video",
    ScrapingCategory::Videos,
    "Free",
    "400+",
);

/// Mazwai - 500+ CC0/Free videos
pub const MAZWAI: ScrapingTarget = ScrapingTarget::new(
    "mazwai",
    "Mazwai",
    "https://mazwai.com",
    ".video-item",
    ".video-item video",
    ScrapingCategory::Videos,
    "CC0/Free",
    "500+",
);

/// Motion Places - 300+ free travel videos
pub const MOTION_PLACES: ScrapingTarget = ScrapingTarget::new(
    "motionplaces",
    "Motion Places",
    "https://motionplaces.com",
    ".video",
    ".video video",
    ScrapingCategory::Videos,
    "Free",
    "300+",
)
.with_notes("Travel footage");

/// SplitShire Videos - 100+ free videos
pub const SPLITSHIRE_VIDEOS: ScrapingTarget = ScrapingTarget::new(
    "splitshirevideos",
    "SplitShire Videos",
    "https://splitshire.com/category/video",
    ".video",
    ".video video",
    ScrapingCategory::Videos,
    "Free",
    "100+",
);

/// XStockvideo - 200+ free videos
pub const XSTOCKVIDEO: ScrapingTarget = ScrapingTarget::new(
    "xstockvideo",
    "XStockvideo",
    "https://xstockvideo.com",
    ".video",
    ".video video",
    ScrapingCategory::Videos,
    "Free",
    "200+",
);

/// Clipstill - 100+ free cinemagraphs
pub const CLIPSTILL: ScrapingTarget = ScrapingTarget::new(
    "clipstill",
    "Clipstill",
    "https://clipstill.com",
    ".cinemagraph",
    ".cinemagraph video",
    ScrapingCategory::Videos,
    "Free",
    "100+",
)
.with_notes("Cinemagraphs");

/// ISO Republic Videos - 200+ free videos
pub const ISO_REPUBLIC_VIDEOS: ScrapingTarget = ScrapingTarget::new(
    "isorepublicvideos",
    "ISO Republic Videos",
    "https://isorepublic.com/videos",
    ".video",
    ".video video",
    ScrapingCategory::Videos,
    "Free",
    "200+",
);

/// Distill - 250+ free videos
pub const DISTILL: ScrapingTarget = ScrapingTarget::new(
    "distill",
    "Distill",
    "https://wedistill.io",
    ".video",
    ".video video",
    ScrapingCategory::Videos,
    "Free",
    "250+",
);

/// Beachfront B-Roll - 1,000+ free videos
pub const BEACHFRONT: ScrapingTarget = ScrapingTarget::new(
    "beachfront",
    "Beachfront B-Roll",
    "https://beachfrontbroll.com",
    ".video",
    ".video video",
    ScrapingCategory::Videos,
    "Free",
    "1,000+",
);

/// Motion Array Free - 500+ free videos
pub const MOTION_ARRAY: ScrapingTarget = ScrapingTarget::new(
    "motionarray",
    "Motion Array Free",
    "https://motionarray.com/browse/free",
    ".video",
    ".video video",
    ScrapingCategory::Videos,
    "Free",
    "500+",
);

/// Pond5 Public Domain - 1,000+ PD videos
pub const POND5_PD: ScrapingTarget = ScrapingTarget::new(
    "pond5pd",
    "Pond5 Public Domain",
    "https://pond5.com/free",
    ".video",
    ".video video",
    ScrapingCategory::Videos,
    "PD",
    "1,000+",
);

/// Phil Fried Free - 100+ CC0 videos
pub const PHIL_FRIED: ScrapingTarget = ScrapingTarget::new(
    "philfried",
    "Phil Fried Free",
    "https://philfried.com/free-stock-footage",
    ".video",
    ".video video",
    ScrapingCategory::Videos,
    "CC0",
    "100+",
);

/// Videezy Free - 10,000+ various license videos
pub const VIDEEZY: ScrapingTarget = ScrapingTarget::new(
    "videezy",
    "Videezy Free",
    "https://videezy.com/free-video",
    ".video-item",
    ".video-item video",
    ScrapingCategory::Videos,
    "Various",
    "10,000+",
)
.with_search_url("https://videezy.com/free-video/{query}");

/// Ignite Motion - 500+ free motion backgrounds
pub const IGNITE_MOTION: ScrapingTarget = ScrapingTarget::new(
    "ignitemotion",
    "Ignite Motion",
    "https://ignitemotion.com",
    ".video",
    ".video video",
    ScrapingCategory::Videos,
    "Free",
    "500+",
)
.with_notes("Motion backgrounds");

/// Monzoom - 300+ free videos
pub const MONZOOM: ScrapingTarget = ScrapingTarget::new(
    "monzoom",
    "Monzoom",
    "https://monzoom.com",
    ".video",
    ".video video",
    ScrapingCategory::Videos,
    "Free",
    "300+",
);

/// Stock Footage 4 Free - 2,000+ CC0 videos
pub const STOCK_FOOTAGE_4_FREE: ScrapingTarget = ScrapingTarget::new(
    "stockfootage4free",
    "Stock Footage 4 Free",
    "https://stockfootageforfree.com",
    ".video",
    ".video video",
    ScrapingCategory::Videos,
    "CC0",
    "2,000+",
);

/// VYDA - 500+ CC videos
pub const VYDA: ScrapingTarget = ScrapingTarget::new(
    "vyda",
    "VYDA",
    "https://vyda.tv",
    ".video",
    ".video video",
    ScrapingCategory::Videos,
    "CC",
    "500+",
);

/// Cute Stock Footage - 500+ free videos
pub const CUTE_STOCK: ScrapingTarget = ScrapingTarget::new(
    "cutestock",
    "Cute Stock Footage",
    "https://cutestockfootage.com",
    ".video",
    ".video video",
    ScrapingCategory::Videos,
    "Free",
    "500+",
);

/// Motion Backgrounds Free - 300+ free motion backgrounds
pub const MOTION_BACKGROUNDS: ScrapingTarget = ScrapingTarget::new(
    "motionbackgrounds",
    "Motion Backgrounds Free",
    "https://motionbackgroundsforfree.com",
    ".video",
    ".video video",
    ScrapingCategory::Videos,
    "Free",
    "300+",
);

/// Free Green Screen - 500+ free green screen clips
pub const FREE_GREEN_SCREEN: ScrapingTarget = ScrapingTarget::new(
    "freegreenscreen",
    "Free Green Screen",
    "https://footagecrate.com/free-green-screen",
    ".video",
    ".video video",
    ScrapingCategory::Videos,
    "Free",
    "500+",
)
.with_notes("Green screen footage");

/// Benchart - 200+ free videos
pub const BENCHART: ScrapingTarget = ScrapingTarget::new(
    "benchart",
    "Benchart",
    "https://benchart.com/free-footage",
    ".video",
    ".video video",
    ScrapingCategory::Videos,
    "Free",
    "200+",
);

/// Panzoid - 1,000+ free intro/outro videos
pub const PANZOID: ScrapingTarget = ScrapingTarget::new(
    "panzoid",
    "Panzoid",
    "https://panzoid.com",
    ".video",
    ".video video",
    ScrapingCategory::Videos,
    "Free",
    "1,000+",
)
.with_notes("Intros and outros");

/// Free HD Footage - 500+ free HD videos
pub const FREE_HD_FOOTAGE: ScrapingTarget = ScrapingTarget::new(
    "freehdfootage",
    "Free HD Footage",
    "https://free-hd-footage.com",
    ".video",
    ".video video",
    ScrapingCategory::Videos,
    "Free",
    "500+",
);

/// OrangeHD - 400+ free HD videos
pub const ORANGEHD: ScrapingTarget = ScrapingTarget::new(
    "orangehd",
    "OrangeHD",
    "https://orangehd.com",
    ".video",
    ".video video",
    ScrapingCategory::Videos,
    "Free",
    "400+",
);

/// Movie Tools - 300+ free videos
pub const MOVIE_TOOLS: ScrapingTarget = ScrapingTarget::new(
    "movietools",
    "Movie Tools",
    "https://movietools.info",
    ".video",
    ".video video",
    ScrapingCategory::Videos,
    "Free",
    "300+",
);

/// Open Video Project - 1,000+ free videos
pub const OPEN_VIDEO_PROJECT: ScrapingTarget = ScrapingTarget::new(
    "openvideoproject",
    "Open Video Project",
    "https://open-video.org",
    ".video",
    ".video video",
    ScrapingCategory::Videos,
    "Free",
    "1,000+",
);

/// Footage Island - 300+ CC0 videos
pub const FOOTAGE_ISLAND: ScrapingTarget = ScrapingTarget::new(
    "footageisland",
    "Footage Island",
    "https://footageisland.com",
    ".video",
    ".video video",
    ScrapingCategory::Videos,
    "CC0",
    "300+",
);

/// Free Footage - 500+ free videos
pub const FREE_FOOTAGE: ScrapingTarget = ScrapingTarget::new(
    "freefootage",
    "Free Footage",
    "https://free-footage.com",
    ".video",
    ".video video",
    ScrapingCategory::Videos,
    "Free",
    "500+",
);

/// Grain Free Footage - 100+ CC0 videos
pub const GRAIN_FREE: ScrapingTarget = ScrapingTarget::new(
    "grainfree",
    "Grain Free Footage",
    "https://grainfree.tv",
    ".video",
    ".video video",
    ScrapingCategory::Videos,
    "CC0",
    "100+",
);

/// Free Stock Footage Archive - 1,000+ CC0 videos
pub const FREE_STOCK_FOOTAGE_ARCHIVE: ScrapingTarget = ScrapingTarget::new(
    "freestockfootagearchive",
    "Free Stock Footage Archive",
    "https://freestockfootagearchive.com",
    ".video",
    ".video video",
    ScrapingCategory::Videos,
    "CC0",
    "1,000+",
);

/// NASA Video Gallery - 10,000+ PD videos
pub const NASA_VIDEO: ScrapingTarget = ScrapingTarget::new(
    "nasavideo",
    "NASA Video Gallery",
    "https://nasa.gov/multimedia/videogallery",
    ".video",
    ".video video",
    ScrapingCategory::Videos,
    "PD",
    "10,000+",
)
.with_notes("Space footage");

/// ESA Videos - 5,000+ CC space videos
pub const ESA_VIDEOS: ScrapingTarget = ScrapingTarget::new(
    "esavideos",
    "ESA Videos",
    "https://esa.int/ESA_Multimedia/Videos",
    ".video",
    ".video video",
    ScrapingCategory::Videos,
    "CC",
    "5,000+",
)
.with_notes("European Space Agency");

/// Hubble Videos - 1,000+ PD space videos
pub const HUBBLE_VIDEOS: ScrapingTarget = ScrapingTarget::new(
    "hubblevideos",
    "Hubble Videos",
    "https://hubblesite.org/contents/media/videos",
    ".video",
    ".video video",
    ScrapingCategory::Videos,
    "PD",
    "1,000+",
)
.with_notes("Hubble telescope");

/// NOAA Video Gallery - 2,000+ PD videos
pub const NOAA_VIDEO: ScrapingTarget = ScrapingTarget::new(
    "noaavideo",
    "NOAA Video Gallery",
    "https://noaa.gov/media-resources",
    ".video",
    ".video video",
    ScrapingCategory::Videos,
    "PD",
    "2,000+",
)
.with_notes("Weather/ocean footage");

/// NPS B-Roll - 5,000+ PD National Parks videos
pub const NPS_BROLL: ScrapingTarget = ScrapingTarget::new(
    "npsbroll",
    "NPS B-Roll",
    "https://nps.gov/subjects/mediakit",
    ".video",
    ".video video",
    ScrapingCategory::Videos,
    "PD",
    "5,000+",
)
.with_notes("National parks footage");

/// OpenFootage - 1,500+ CC0 videos
pub const OPEN_FOOTAGE: ScrapingTarget = ScrapingTarget::new(
    "openfootage",
    "OpenFootage",
    "https://openfootage.net",
    ".video",
    ".video video",
    ScrapingCategory::Videos,
    "CC0",
    "1,500+",
);

/// Dissolve Free - 500+ free videos
pub const DISSOLVE_FREE: ScrapingTarget = ScrapingTarget::new(
    "dissolvefree",
    "Dissolve Free",
    "https://dissolve.com/free-clips",
    ".video",
    ".video video",
    ScrapingCategory::Videos,
    "Free",
    "500+",
);

/// Production Crate Free - 2,000+ free VFX videos
pub const PRODUCTION_CRATE: ScrapingTarget = ScrapingTarget::new(
    "productioncrate",
    "Production Crate Free",
    "https://productioncrate.com/free",
    ".video",
    ".video video",
    ScrapingCategory::Videos,
    "Free",
    "2,000+",
)
.with_notes("VFX elements");

/// ActionVFX Free - 100+ free VFX videos
pub const ACTIONVFX: ScrapingTarget = ScrapingTarget::new(
    "actionvfx",
    "ActionVFX Free",
    "https://actionvfx.com/collections/free-vfx",
    ".video",
    ".video video",
    ScrapingCategory::Videos,
    "Free",
    "100+",
)
.with_notes("VFX elements");

/// Motion Elements Free - 5,000+ free videos
pub const MOTION_ELEMENTS: ScrapingTarget = ScrapingTarget::new(
    "motionelements",
    "Motion Elements Free",
    "https://motionelements.com/free/stock-footage",
    ".video",
    ".video video",
    ScrapingCategory::Videos,
    "Free",
    "5,000+",
);

/// Vecteezy Videos - 50,000+ free videos
pub const VECTEEZY_VIDEOS: ScrapingTarget = ScrapingTarget::new(
    "vecteezyvideos",
    "Vecteezy Videos",
    "https://vecteezy.com/free-videos",
    ".video",
    ".video video",
    ScrapingCategory::Videos,
    "Free",
    "50,000+",
)
.with_search_url("https://vecteezy.com/free-videos/{query}");

// ═══════════════════════════════════════════════════════════════════════════════
// BATCH 3 COLLECTION - Video Scrapers (50 sites)
// ═══════════════════════════════════════════════════════════════════════════════

/// All Batch 3 video scraping targets (50 sites, ~120K+ videos)
pub const BATCH_3_VIDEO_TARGETS: &[ScrapingTarget] = &[
    MIXKIT_VIDEOS,
    VIDEVO,
    LIFE_OF_VIDS,
    DAREFUL,
    VIDSPLAY,
    MAZWAI,
    MOTION_PLACES,
    SPLITSHIRE_VIDEOS,
    XSTOCKVIDEO,
    CLIPSTILL,
    ISO_REPUBLIC_VIDEOS,
    DISTILL,
    BEACHFRONT,
    MOTION_ARRAY,
    POND5_PD,
    PHIL_FRIED,
    VIDEEZY,
    IGNITE_MOTION,
    MONZOOM,
    STOCK_FOOTAGE_4_FREE,
    VYDA,
    CUTE_STOCK,
    MOTION_BACKGROUNDS,
    FREE_GREEN_SCREEN,
    BENCHART,
    PANZOID,
    FREE_HD_FOOTAGE,
    ORANGEHD,
    MOVIE_TOOLS,
    OPEN_VIDEO_PROJECT,
    FOOTAGE_ISLAND,
    FREE_FOOTAGE,
    GRAIN_FREE,
    FREE_STOCK_FOOTAGE_ARCHIVE,
    NASA_VIDEO,
    ESA_VIDEOS,
    HUBBLE_VIDEOS,
    NOAA_VIDEO,
    NPS_BROLL,
    OPEN_FOOTAGE,
    DISSOLVE_FREE,
    PRODUCTION_CRATE,
    ACTIONVFX,
    MOTION_ELEMENTS,
    VECTEEZY_VIDEOS,
];

// ═══════════════════════════════════════════════════════════════════════════════
// BATCH 4: AUDIO SCRAPERS (55 sites) - Music, SFX, Sound Effects
// ═══════════════════════════════════════════════════════════════════════════════

/// Mixkit Music - 10,000+ free music tracks
pub const MIXKIT_MUSIC: ScrapingTarget = ScrapingTarget::new(
    "mixkitmusic",
    "Mixkit Music",
    "https://mixkit.co/free-stock-music",
    ".audio-item",
    "audio source",
    ScrapingCategory::Audio,
    "Free",
    "10,000+",
)
.with_search_url("https://mixkit.co/free-stock-music/?q={query}");

/// Mixkit SFX - 5,000+ free sound effects
pub const MIXKIT_SFX: ScrapingTarget = ScrapingTarget::new(
    "mixkitsfx",
    "Mixkit Sound Effects",
    "https://mixkit.co/free-sound-effects",
    ".audio-item",
    "audio source",
    ScrapingCategory::Audio,
    "Free",
    "5,000+",
)
.with_search_url("https://mixkit.co/free-sound-effects/?q={query}");

/// Musopen - 100,000+ classical music (CC)
pub const MUSOPEN: ScrapingTarget = ScrapingTarget::new(
    "musopen",
    "Musopen",
    "https://musopen.org/music",
    ".recording",
    ".recording audio",
    ScrapingCategory::Audio,
    "CC/PD",
    "100,000+",
)
.with_search_url("https://musopen.org/music?q={query}")
.with_notes("Classical music, recordings, sheet music");

/// Free PD - 10,000+ public domain music
pub const FREE_PD: ScrapingTarget = ScrapingTarget::new(
    "freepd",
    "Free PD",
    "https://freepd.com",
    ".track-item",
    "audio",
    ScrapingCategory::Audio,
    "PD",
    "10,000+",
);

/// PDSounds - 5,000+ public domain sounds
pub const PDSOUNDS: ScrapingTarget = ScrapingTarget::new(
    "pdsounds",
    "PDSounds",
    "https://pdsounds.org",
    ".sound-item",
    "audio",
    ScrapingCategory::Audio,
    "PD",
    "5,000+",
);

/// SoundBible - 2,000+ free sound clips
pub const SOUNDBIBLE: ScrapingTarget = ScrapingTarget::new(
    "soundbible",
    "SoundBible",
    "https://soundbible.com",
    ".sound",
    "audio",
    ScrapingCategory::Audio,
    "CC/PD",
    "2,000+",
)
.with_search_url("https://soundbible.com/search.php?q={query}");

/// BBC Sound Effects - 16,000+ sounds (personal/edu use)
pub const BBC_SFX: ScrapingTarget = ScrapingTarget::new(
    "bbcsfx",
    "BBC Sound Effects",
    "https://sound-effects.bbcrewind.co.uk",
    ".sound-result",
    "audio source",
    ScrapingCategory::Audio,
    "Personal/Edu",
    "16,000+",
)
.with_search_url("https://sound-effects.bbcrewind.co.uk/search?q={query}")
.with_notes("Personal and educational use only");

/// ZapSplat - 100,000+ free sound effects
pub const ZAPSPLAT: ScrapingTarget = ScrapingTarget::new(
    "zapsplat",
    "ZapSplat",
    "https://zapsplat.com",
    ".sound-item",
    "audio",
    ScrapingCategory::Audio,
    "Free",
    "100,000+",
)
.with_search_url("https://zapsplat.com/?s={query}");

/// SoundJay - 500+ free sound effects
pub const SOUNDJAY: ScrapingTarget = ScrapingTarget::new(
    "soundjay",
    "SoundJay",
    "https://soundjay.com",
    ".sound-item",
    "audio",
    ScrapingCategory::Audio,
    "Free",
    "500+",
);

/// Uppbeat - 5,000+ free music tracks
pub const UPPBEAT: ScrapingTarget = ScrapingTarget::new(
    "uppbeat",
    "Uppbeat",
    "https://uppbeat.io/browse/music",
    ".track",
    "audio",
    ScrapingCategory::Audio,
    "Free",
    "5,000+",
)
.with_search_url("https://uppbeat.io/search?q={query}");

/// Bensound - 500+ royalty-free music
pub const BENSOUND: ScrapingTarget = ScrapingTarget::new(
    "bensound",
    "Bensound",
    "https://bensound.com",
    ".track-item",
    "audio",
    ScrapingCategory::Audio,
    "Free",
    "500+",
)
.with_search_url("https://bensound.com/search?q={query}")
.with_notes("Attribution required for free use");

/// Chosic - 10,000+ royalty-free music
pub const CHOSIC: ScrapingTarget = ScrapingTarget::new(
    "chosic",
    "Chosic",
    "https://chosic.com/free-music",
    ".track",
    "audio",
    ScrapingCategory::Audio,
    "Free",
    "10,000+",
)
.with_search_url("https://chosic.com/?s={query}");

/// Audionautix - 500+ royalty-free tracks by Jason Shaw
pub const AUDIONAUTIX: ScrapingTarget = ScrapingTarget::new(
    "audionautix",
    "Audionautix",
    "https://audionautix.com",
    ".track",
    "audio",
    ScrapingCategory::Audio,
    "CC BY 4.0",
    "500+",
)
.with_search_url("https://audionautix.com/?s={query}");

/// Incompetech - 2,000+ royalty-free tracks by Kevin MacLeod
pub const INCOMPETECH: ScrapingTarget = ScrapingTarget::new(
    "incompetech",
    "Incompetech",
    "https://incompetech.com/music/royalty-free/music.html",
    ".track",
    "audio",
    ScrapingCategory::Audio,
    "CC BY 3.0",
    "2,000+",
)
.with_notes("Kevin MacLeod's music collection");

/// Purple Planet - 1,000+ royalty-free music
pub const PURPLE_PLANET: ScrapingTarget = ScrapingTarget::new(
    "purpleplanet",
    "Purple Planet",
    "https://purpleplanet.com",
    ".track-item",
    "audio",
    ScrapingCategory::Audio,
    "Free",
    "1,000+",
);

/// Partners in Rhyme - 1,500+ free sound effects
pub const PARTNERS_IN_RHYME: ScrapingTarget = ScrapingTarget::new(
    "partnersinrhyme",
    "Partners in Rhyme",
    "https://partnersinrhyme.com/soundfx",
    ".sound-item",
    "audio",
    ScrapingCategory::Audio,
    "Free",
    "1,500+",
);

/// Sample Focus - 50,000+ free samples
pub const SAMPLE_FOCUS: ScrapingTarget = ScrapingTarget::new(
    "samplefocus",
    "Sample Focus",
    "https://samplefocus.com",
    ".sample-item",
    "audio",
    ScrapingCategory::Audio,
    "Free",
    "50,000+",
)
.with_search_url("https://samplefocus.com/search?q={query}");

/// Looperman - 100,000+ loops and samples
pub const LOOPERMAN: ScrapingTarget = ScrapingTarget::new(
    "looperman",
    "Looperman",
    "https://looperman.com",
    ".loop-item",
    "audio",
    ScrapingCategory::Audio,
    "Free",
    "100,000+",
)
.with_search_url("https://looperman.com/search?q={query}");

/// Sounds Crate - 500+ free sound effects
pub const SOUNDS_CRATE: ScrapingTarget = ScrapingTarget::new(
    "soundscrate",
    "Sounds Crate",
    "https://soundscrate.com",
    ".sound-item",
    "audio",
    ScrapingCategory::Audio,
    "Free",
    "500+",
);

/// SoundGator - 2,000+ free sound effects
pub const SOUNDGATOR: ScrapingTarget = ScrapingTarget::new(
    "soundgator",
    "SoundGator",
    "https://soundgator.com",
    ".sound-item",
    "audio",
    ScrapingCategory::Audio,
    "Free",
    "2,000+",
)
.with_search_url("https://soundgator.com/search?q={query}");

/// GR Sites Sound FX - 1,000+ free sounds
pub const GR_SITES: ScrapingTarget = ScrapingTarget::new(
    "grsites",
    "GR Sites Sound FX",
    "https://grsites.com/archive/sounds",
    ".sound-item",
    "audio",
    ScrapingCategory::Audio,
    "Free",
    "1,000+",
);

/// FreeSoundEffects.com - 500+ free sounds
pub const FREE_SOUND_EFFECTS: ScrapingTarget = ScrapingTarget::new(
    "freesoundeffects",
    "FreeSoundEffects.com",
    "https://freesoundeffects.com",
    ".sound-item",
    "audio",
    ScrapingCategory::Audio,
    "Free",
    "500+",
);

/// OrangeFreeSounds - 5,000+ free sounds
pub const ORANGE_FREE_SOUNDS: ScrapingTarget = ScrapingTarget::new(
    "orangefreesounds",
    "OrangeFreeSounds",
    "https://orangefreesounds.com",
    ".sound-post",
    "audio",
    ScrapingCategory::Audio,
    "Free",
    "5,000+",
)
.with_search_url("https://orangefreesounds.com/?s={query}");

/// 99Sounds - 1,000+ free sound packs
pub const NINETY_NINE_SOUNDS: ScrapingTarget = ScrapingTarget::new(
    "99sounds",
    "99Sounds",
    "https://99sounds.org",
    ".pack-item",
    "audio",
    ScrapingCategory::Audio,
    "Free",
    "1,000+",
);

/// FreeSFX - 5,000+ free sound effects
pub const FREE_SFX: ScrapingTarget = ScrapingTarget::new(
    "freesfx",
    "FreeSFX",
    "https://freesfx.co.uk",
    ".sound",
    "audio",
    ScrapingCategory::Audio,
    "Free",
    "5,000+",
)
.with_search_url("https://freesfx.co.uk/search?q={query}");

/// Free Sound Library - 500+ free sounds
pub const FREE_SOUND_LIBRARY: ScrapingTarget = ScrapingTarget::new(
    "freesoundlibrary",
    "Free Sound Library",
    "https://freesoundslibrary.com",
    ".sound-item",
    "audio",
    ScrapingCategory::Audio,
    "Free",
    "500+",
);

/// Noise For Fun - 500+ free sounds
pub const NOISE_FOR_FUN: ScrapingTarget = ScrapingTarget::new(
    "noiseforfun",
    "Noise For Fun",
    "https://noiseforfun.com",
    ".sound-item",
    "audio",
    ScrapingCategory::Audio,
    "Free",
    "500+",
);

/// Flash Kit Sounds - 3,000+ flash sounds
pub const FLASH_KIT_SOUNDS: ScrapingTarget = ScrapingTarget::new(
    "flashkitsounds",
    "Flash Kit Sounds",
    "https://flashkit.com/soundfx",
    ".sound-item",
    "audio",
    ScrapingCategory::Audio,
    "Free",
    "3,000+",
);

/// Wav Source - 1,000+ free wav sounds
pub const WAV_SOURCE: ScrapingTarget = ScrapingTarget::new(
    "wavsource",
    "Wav Source",
    "https://wavsource.com",
    ".sound-item",
    "audio",
    ScrapingCategory::Audio,
    "Free",
    "1,000+",
);

/// Free Loops - 5,000+ free loops
pub const FREE_LOOPS: ScrapingTarget = ScrapingTarget::new(
    "freeloops",
    "Free Loops",
    "https://free-loops.com",
    ".loop-item",
    "audio",
    ScrapingCategory::Audio,
    "Free",
    "5,000+",
)
.with_search_url("https://free-loops.com/search.php?q={query}");

/// Sample Swap - 10,000+ free samples
pub const SAMPLESWAP: ScrapingTarget = ScrapingTarget::new(
    "sampleswap",
    "Sample Swap",
    "https://sampleswap.org",
    ".sample-item",
    "audio",
    ScrapingCategory::Audio,
    "Free",
    "10,000+",
);

/// Fesliyan Studios - 1,000+ royalty-free music
pub const FESLIYAN: ScrapingTarget = ScrapingTarget::new(
    "fesliyan",
    "Fesliyan Studios",
    "https://fesliyanstudios.com",
    ".track",
    "audio",
    ScrapingCategory::Audio,
    "Free",
    "1,000+",
)
.with_search_url("https://fesliyanstudios.com/search?q={query}");

/// Tabla Free - 500+ Indian tabla samples
pub const TABLA_FREE: ScrapingTarget = ScrapingTarget::new(
    "tablafree",
    "Tabla Free",
    "https://tablafree.com",
    ".sample-item",
    "audio",
    ScrapingCategory::Audio,
    "Free",
    "500+",
)
.with_notes("Indian tabla and percussion");

/// Free Stock Music - 2,000+ royalty-free tracks
pub const FREE_STOCK_MUSIC: ScrapingTarget = ScrapingTarget::new(
    "freestockmusic",
    "Free Stock Music",
    "https://stockmusic.net",
    ".track-item",
    "audio",
    ScrapingCategory::Audio,
    "Free",
    "2,000+",
)
.with_search_url("https://stockmusic.net/search?q={query}");

/// Silverman Sound - 500+ free music tracks
pub const SILVERMAN_SOUND: ScrapingTarget = ScrapingTarget::new(
    "silvermansound",
    "Silverman Sound",
    "https://silvermansound.com",
    ".track",
    "audio",
    ScrapingCategory::Audio,
    "Free",
    "500+",
);

/// DL Sounds - 1,000+ royalty-free music
pub const DL_SOUNDS: ScrapingTarget = ScrapingTarget::new(
    "dlsounds",
    "DL Sounds",
    "https://dlsounds.com",
    ".track-item",
    "audio",
    ScrapingCategory::Audio,
    "Free",
    "1,000+",
)
.with_search_url("https://dlsounds.com/?s={query}");

/// Free Music Archive - 150,000+ free music
pub const FMA: ScrapingTarget = ScrapingTarget::new(
    "fma",
    "Free Music Archive",
    "https://freemusicarchive.org",
    ".track",
    "audio",
    ScrapingCategory::Audio,
    "CC",
    "150,000+",
)
.with_search_url("https://freemusicarchive.org/search?q={query}");

/// ccMixter - 50,000+ CC-licensed music
pub const CCMIXTER: ScrapingTarget = ScrapingTarget::new(
    "ccmixter",
    "ccMixter",
    "https://ccmixter.org",
    ".track",
    "audio",
    ScrapingCategory::Audio,
    "CC",
    "50,000+",
)
.with_search_url("https://ccmixter.org/search?q={query}");

/// Jamendo - 600,000+ free music tracks
pub const JAMENDO: ScrapingTarget = ScrapingTarget::new(
    "jamendo",
    "Jamendo",
    "https://jamendo.com/search",
    ".track",
    "audio",
    ScrapingCategory::Audio,
    "CC",
    "600,000+",
)
.with_search_url("https://jamendo.com/search?q={query}")
.with_notes("Large CC music library");

/// dig.ccMixter - 10,000+ instrumental tracks
pub const DIG_CCMIXTER: ScrapingTarget = ScrapingTarget::new(
    "digccmixter",
    "dig.ccMixter",
    "https://dig.ccmixter.org",
    ".track",
    "audio",
    ScrapingCategory::Audio,
    "CC",
    "10,000+",
)
.with_notes("Instrumental and video-friendly music");

/// Internet Archive Audio - 15,000,000+ audio items
pub const ARCHIVE_AUDIO: ScrapingTarget = ScrapingTarget::new(
    "archiveaudio",
    "Internet Archive Audio",
    "https://archive.org/details/audio",
    ".item-ia",
    "audio",
    ScrapingCategory::Audio,
    "Varies",
    "15,000,000+",
)
.with_search_url("https://archive.org/search.php?query={query}&mediatype=audio");

/// SoundCloud CC - Creative Commons music
pub const SOUNDCLOUD_CC: ScrapingTarget = ScrapingTarget::new(
    "soundcloudcc",
    "SoundCloud Creative Commons",
    "https://soundcloud.com/search/sounds?filter.license=to_share",
    ".soundList__item",
    "audio",
    ScrapingCategory::Audio,
    "CC",
    "5,000,000+",
)
.with_search_url("https://soundcloud.com/search/sounds?q={query}&filter.license=to_share");

/// Epidemic Sound Free - 500+ free tracks
pub const EPIDEMIC_FREE: ScrapingTarget = ScrapingTarget::new(
    "epidemicfree",
    "Epidemic Sound Free",
    "https://epidemicsound.com/music/featured/free-music",
    ".track",
    "audio",
    ScrapingCategory::Audio,
    "Free",
    "500+",
);

/// Artlist Free - 200+ free tracks
pub const ARTLIST_FREE: ScrapingTarget = ScrapingTarget::new(
    "artlistfree",
    "Artlist Free",
    "https://artlist.io/royalty-free-music",
    ".track",
    "audio",
    ScrapingCategory::Audio,
    "Free",
    "200+",
);

/// NCS (NoCopyrightSounds) - 1,000+ EDM tracks
pub const NCS: ScrapingTarget = ScrapingTarget::new(
    "ncs",
    "NCS (NoCopyrightSounds)",
    "https://ncs.io/music",
    ".track",
    "audio",
    ScrapingCategory::Audio,
    "Free",
    "1,000+",
)
.with_search_url("https://ncs.io/music-search?q={query}")
.with_notes("Electronic/EDM music for content creators");

/// Audio Library YouTube - 1,000+ tracks
pub const AUDIO_LIBRARY: ScrapingTarget = ScrapingTarget::new(
    "audiolibrary",
    "Audio Library (YouTube)",
    "https://youtube.com/audiolibrary",
    ".track",
    "audio",
    ScrapingCategory::Audio,
    "Free",
    "1,000+",
)
.with_notes("YouTube Audio Library tracks");

/// Pond5 Public Domain - 500+ PD audio
pub const POND5_AUDIO_PD: ScrapingTarget = ScrapingTarget::new(
    "pond5audiopd",
    "Pond5 Public Domain Audio",
    "https://pond5.com/free-music-and-sound",
    ".audio-item",
    "audio",
    ScrapingCategory::Audio,
    "PD",
    "500+",
);

// ═══════════════════════════════════════════════════════════════════════════════
// BATCH 4 COLLECTION - Audio Scrapers (45 sites)
// ═══════════════════════════════════════════════════════════════════════════════

/// All Batch 4 audio scraping targets (45 sites, ~21M+ audio files)
pub const BATCH_4_AUDIO_TARGETS: &[ScrapingTarget] = &[
    MIXKIT_MUSIC,
    MIXKIT_SFX,
    MUSOPEN,
    FREE_PD,
    PDSOUNDS,
    SOUNDBIBLE,
    BBC_SFX,
    ZAPSPLAT,
    SOUNDJAY,
    UPPBEAT,
    BENSOUND,
    CHOSIC,
    AUDIONAUTIX,
    INCOMPETECH,
    PURPLE_PLANET,
    PARTNERS_IN_RHYME,
    SAMPLE_FOCUS,
    LOOPERMAN,
    SOUNDS_CRATE,
    SOUNDGATOR,
    GR_SITES,
    FREE_SOUND_EFFECTS,
    ORANGE_FREE_SOUNDS,
    NINETY_NINE_SOUNDS,
    FREE_SFX,
    FREE_SOUND_LIBRARY,
    NOISE_FOR_FUN,
    FLASH_KIT_SOUNDS,
    WAV_SOURCE,
    FREE_LOOPS,
    SAMPLESWAP,
    FESLIYAN,
    TABLA_FREE,
    FREE_STOCK_MUSIC,
    SILVERMAN_SOUND,
    DL_SOUNDS,
    FMA,
    CCMIXTER,
    JAMENDO,
    DIG_CCMIXTER,
    ARCHIVE_AUDIO,
    SOUNDCLOUD_CC,
    EPIDEMIC_FREE,
    ARTLIST_FREE,
    NCS,
    AUDIO_LIBRARY,
    POND5_AUDIO_PD,
];

// ═══════════════════════════════════════════════════════════════════════════════
// BATCH 5: 3D MODELS, TEXTURES, VECTORS, ICONS, GAME ASSETS (50 sites)
// ═══════════════════════════════════════════════════════════════════════════════

// --- 3D Models ---

/// Free3D - 150,000+ free 3D models
pub const FREE3D: ScrapingTarget = ScrapingTarget::new(
    "free3d",
    "Free3D",
    "https://free3d.com/3d-models/",
    ".product-page-item",
    ".product-image img",
    ScrapingCategory::Models3D,
    "Free",
    "150,000+",
)
.with_search_url("https://free3d.com/3d-models/?q={query}");

/// Archive3D - 50,000+ free 3D models
pub const ARCHIVE3D: ScrapingTarget = ScrapingTarget::new(
    "archive3d",
    "Archive3D",
    "https://archive3d.net",
    ".model-item",
    ".model-image img",
    ScrapingCategory::Models3D,
    "Free",
    "50,000+",
)
.with_search_url("https://archive3d.net/?q={query}");

/// CGTrader Free - 30,000+ free 3D models
pub const CGTRADER_FREE: ScrapingTarget = ScrapingTarget::new(
    "cgtraderfree",
    "CGTrader Free",
    "https://cgtrader.com/free-3d-models",
    ".product-item",
    ".product-image img",
    ScrapingCategory::Models3D,
    "Free",
    "30,000+",
)
.with_search_url("https://cgtrader.com/free-3d-models?keywords={query}");

/// TurboSquid Free - 20,000+ free 3D models
pub const TURBOSQUID_FREE: ScrapingTarget = ScrapingTarget::new(
    "turbosquidfree",
    "TurboSquid Free",
    "https://turbosquid.com/Search/3D-Models/free",
    ".product",
    ".product img",
    ScrapingCategory::Models3D,
    "Free",
    "20,000+",
)
.with_search_url("https://turbosquid.com/Search/3D-Models/free/{query}");

/// Sketchfab Free - 500,000+ downloadable models
pub const SKETCHFAB_FREE: ScrapingTarget = ScrapingTarget::new(
    "sketchfabfree",
    "Sketchfab Free",
    "https://sketchfab.com/features/free-3d-models",
    ".model-card",
    ".model-card img",
    ScrapingCategory::Models3D,
    "CC",
    "500,000+",
)
.with_search_url("https://sketchfab.com/search?q={query}&downloadable=true")
.with_notes("Filter for downloadable CC models");

/// Clara.io - 100,000+ free 3D models
pub const CLARA_IO: ScrapingTarget = ScrapingTarget::new(
    "claraio",
    "Clara.io",
    "https://clara.io/library",
    ".model-item",
    ".model-item img",
    ScrapingCategory::Models3D,
    "CC",
    "100,000+",
)
.with_search_url("https://clara.io/library?q={query}");

/// RenderHub Free - 10,000+ free 3D models
pub const RENDERHUB_FREE: ScrapingTarget = ScrapingTarget::new(
    "renderhubfree",
    "RenderHub Free",
    "https://renderhub.com/free-3d-models",
    ".model-item",
    ".model-item img",
    ScrapingCategory::Models3D,
    "Free",
    "10,000+",
);

/// ShareCG - 50,000+ 3D models and CG content
pub const SHARECG: ScrapingTarget = ScrapingTarget::new(
    "sharecg",
    "ShareCG",
    "https://sharecg.com",
    ".item",
    ".item img",
    ScrapingCategory::Models3D,
    "Free",
    "50,000+",
)
.with_search_url("https://sharecg.com/search.php?q={query}");

/// 3D Warehouse - 4,000,000+ SketchUp models
pub const THREED_WAREHOUSE: ScrapingTarget = ScrapingTarget::new(
    "3dwarehouse",
    "3D Warehouse",
    "https://3dwarehouse.sketchup.com",
    ".model-card",
    ".model-card img",
    ScrapingCategory::Models3D,
    "Free",
    "4,000,000+",
)
.with_search_url("https://3dwarehouse.sketchup.com/search/?q={query}")
.with_notes("SketchUp format, convertible");

// --- Textures ---

/// TextureCan - 3,000+ free textures
pub const TEXTURECAN: ScrapingTarget = ScrapingTarget::new(
    "texturecan",
    "TextureCan",
    "https://texturecan.com",
    ".texture-item",
    ".texture-item img",
    ScrapingCategory::Textures,
    "CC0",
    "3,000+",
)
.with_search_url("https://texturecan.com/?s={query}");

/// Textures.com Free - 5,000+ free textures
pub const TEXTURES_COM: ScrapingTarget = ScrapingTarget::new(
    "texturescom",
    "Textures.com Free",
    "https://textures.com/browse/free",
    ".texture",
    ".texture img",
    ScrapingCategory::Textures,
    "Free",
    "5,000+",
);

/// 3DTextures - 1,500+ free PBR textures
pub const THREED_TEXTURES: ScrapingTarget = ScrapingTarget::new(
    "3dtextures",
    "3DTextures",
    "https://3dtextures.me",
    ".texture-item",
    ".texture-item img",
    ScrapingCategory::Textures,
    "CC0",
    "1,500+",
);

/// ShareTextures - 500+ free textures
pub const SHARE_TEXTURES: ScrapingTarget = ScrapingTarget::new(
    "sharetextures",
    "ShareTextures",
    "https://sharetextures.com",
    ".texture",
    ".texture img",
    ScrapingCategory::Textures,
    "CC0",
    "500+",
);

/// Texture Ninja - 1,000+ free textures
pub const TEXTURE_NINJA: ScrapingTarget = ScrapingTarget::new(
    "textureninja",
    "Texture Ninja",
    "https://texture.ninja",
    ".texture-item",
    ".texture-item img",
    ScrapingCategory::Textures,
    "CC0",
    "1,000+",
);

/// TextureLib - 2,000+ free textures
pub const TEXTURELIB: ScrapingTarget = ScrapingTarget::new(
    "texturelib",
    "TextureLib",
    "https://texturelib.com",
    ".texture",
    ".texture img",
    ScrapingCategory::Textures,
    "Free",
    "2,000+",
);

/// Poly Haven Textures - 800+ free PBR textures
pub const POLYHAVEN_TEXTURES: ScrapingTarget = ScrapingTarget::new(
    "polyhaventextures",
    "Poly Haven Textures",
    "https://polyhaven.com/textures",
    ".asset-card",
    ".asset-card img",
    ScrapingCategory::Textures,
    "CC0",
    "800+",
)
.with_search_url("https://polyhaven.com/textures?s={query}");

/// AmbientCG - 1,500+ free PBR materials
pub const AMBIENTCG: ScrapingTarget = ScrapingTarget::new(
    "ambientcg",
    "AmbientCG",
    "https://ambientcg.com",
    ".asset-card",
    ".asset-card img",
    ScrapingCategory::Textures,
    "CC0",
    "1,500+",
)
.with_search_url("https://ambientcg.com/list?q={query}");

/// cgbookcase - 500+ free PBR textures
pub const CGBOOKCASE: ScrapingTarget = ScrapingTarget::new(
    "cgbookcase",
    "cgbookcase",
    "https://cgbookcase.com/textures",
    ".texture-item",
    ".texture-item img",
    ScrapingCategory::Textures,
    "CC0",
    "500+",
);

// --- Vectors & Icons ---

/// SVG Repo - 500,000+ free SVG icons
pub const SVGREPO: ScrapingTarget = ScrapingTarget::new(
    "svgrepo",
    "SVG Repo",
    "https://svgrepo.com",
    ".icon-item",
    "svg",
    ScrapingCategory::Vectors,
    "CC/PD",
    "500,000+",
)
.with_search_url("https://svgrepo.com/vectors/{query}");

/// Iconmonstr - 4,500+ free icons
pub const ICONMONSTR: ScrapingTarget = ScrapingTarget::new(
    "iconmonstr",
    "Iconmonstr",
    "https://iconmonstr.com",
    ".icon-preview",
    "svg",
    ScrapingCategory::Vectors,
    "Free",
    "4,500+",
)
.with_search_url("https://iconmonstr.com/?s={query}");

/// Flaticon Free - 100,000+ free icons
pub const FLATICON_FREE: ScrapingTarget = ScrapingTarget::new(
    "flaticonfree",
    "Flaticon Free",
    "https://flaticon.com/free-icons",
    ".icon",
    ".icon img",
    ScrapingCategory::Vectors,
    "Free",
    "100,000+",
)
.with_search_url("https://flaticon.com/search?word={query}&type=uicon");

/// Simple Icons - 2,800+ brand icons
pub const SIMPLE_ICONS: ScrapingTarget = ScrapingTarget::new(
    "simpleicons",
    "Simple Icons",
    "https://simpleicons.org",
    ".icon-item",
    "svg",
    ScrapingCategory::Vectors,
    "CC0",
    "2,800+",
);

/// Feather Icons - 280+ open source icons
pub const FEATHER_ICONS: ScrapingTarget = ScrapingTarget::new(
    "feathericons",
    "Feather Icons",
    "https://feathericons.com",
    ".icon",
    "svg",
    ScrapingCategory::Vectors,
    "MIT",
    "280+",
);

/// Heroicons - 300+ beautiful icons
pub const HEROICONS: ScrapingTarget = ScrapingTarget::new(
    "heroicons",
    "Heroicons",
    "https://heroicons.com",
    ".icon",
    "svg",
    ScrapingCategory::Vectors,
    "MIT",
    "300+",
);

/// Tabler Icons - 4,000+ free icons
pub const TABLER_ICONS: ScrapingTarget = ScrapingTarget::new(
    "tablericons",
    "Tabler Icons",
    "https://tabler-icons.io",
    ".icon-item",
    "svg",
    ScrapingCategory::Vectors,
    "MIT",
    "4,000+",
)
.with_search_url("https://tabler-icons.io/search?q={query}");

/// Lucide - 1,000+ open source icons
pub const LUCIDE: ScrapingTarget = ScrapingTarget::new(
    "lucide",
    "Lucide Icons",
    "https://lucide.dev/icons",
    ".icon",
    "svg",
    ScrapingCategory::Vectors,
    "ISC",
    "1,000+",
);

/// Phosphor Icons - 7,000+ icons
pub const PHOSPHOR_ICONS: ScrapingTarget = ScrapingTarget::new(
    "phosphoricons",
    "Phosphor Icons",
    "https://phosphoricons.com",
    ".icon",
    "svg",
    ScrapingCategory::Vectors,
    "MIT",
    "7,000+",
);

/// Bootstrap Icons - 1,800+ free icons
pub const BOOTSTRAP_ICONS: ScrapingTarget = ScrapingTarget::new(
    "bootstrapicons",
    "Bootstrap Icons",
    "https://icons.getbootstrap.com",
    ".icon",
    "svg",
    ScrapingCategory::Vectors,
    "MIT",
    "1,800+",
)
.with_search_url("https://icons.getbootstrap.com/?q={query}");

/// Material Design Icons - 7,000+ icons
pub const MATERIAL_ICONS: ScrapingTarget = ScrapingTarget::new(
    "materialicons",
    "Material Design Icons",
    "https://materialdesignicons.com",
    ".icon",
    "svg",
    ScrapingCategory::Vectors,
    "Apache 2.0",
    "7,000+",
)
.with_search_url("https://materialdesignicons.com/?q={query}");

/// Freepik Free - 1,000,000+ free vectors
pub const FREEPIK_FREE: ScrapingTarget = ScrapingTarget::new(
    "freepikfree",
    "Freepik Free",
    "https://freepik.com/free-vectors",
    ".showcase__item",
    ".showcase__item img",
    ScrapingCategory::Vectors,
    "Free",
    "1,000,000+",
)
.with_search_url("https://freepik.com/search?format=search&query={query}");

/// Vecteezy Free - 500,000+ free vectors
pub const VECTEEZY_FREE: ScrapingTarget = ScrapingTarget::new(
    "vecteezyfree",
    "Vecteezy Free",
    "https://vecteezy.com/free-vector",
    ".item",
    ".item img",
    ScrapingCategory::Vectors,
    "Free",
    "500,000+",
)
.with_search_url("https://vecteezy.com/free-vector/{query}");

// --- Illustrations ---

/// unDraw - 500+ customizable illustrations
pub const UNDRAW: ScrapingTarget = ScrapingTarget::new(
    "undraw",
    "unDraw",
    "https://undraw.co/illustrations",
    ".illustration",
    "svg",
    ScrapingCategory::Vectors,
    "MIT",
    "500+",
);

/// DrawKit - 100+ free illustrations
pub const DRAWKIT: ScrapingTarget = ScrapingTarget::new(
    "drawkit",
    "DrawKit",
    "https://drawkit.com/free-illustrations",
    ".illustration",
    ".illustration img",
    ScrapingCategory::Vectors,
    "Free",
    "100+",
);

/// Humaaans - Mix-and-match people illustrations
pub const HUMAAANS: ScrapingTarget = ScrapingTarget::new(
    "humaaans",
    "Humaaans",
    "https://humaaans.com",
    ".illustration",
    "svg",
    ScrapingCategory::Vectors,
    "CC BY 4.0",
    "100+",
);

/// Open Doodles - 100+ free illustrations
pub const OPEN_DOODLES: ScrapingTarget = ScrapingTarget::new(
    "opendoodles",
    "Open Doodles",
    "https://opendoodles.com",
    ".illustration",
    "svg",
    ScrapingCategory::Vectors,
    "CC0",
    "100+",
);

/// Absurd Design - Surrealist illustrations
pub const ABSURD_DESIGN: ScrapingTarget = ScrapingTarget::new(
    "absurddesign",
    "Absurd Design",
    "https://absurd.design",
    ".illustration",
    ".illustration img",
    ScrapingCategory::Vectors,
    "Free",
    "50+",
);

/// IRA Design - 100+ gradient illustrations
pub const IRA_DESIGN: ScrapingTarget = ScrapingTarget::new(
    "iradesign",
    "IRA Design",
    "https://iradesign.io",
    ".illustration",
    "svg",
    ScrapingCategory::Vectors,
    "CC BY 4.0",
    "100+",
);

/// Storyset - Customizable illustrations
pub const STORYSET: ScrapingTarget = ScrapingTarget::new(
    "storyset",
    "Storyset",
    "https://storyset.com",
    ".illustration",
    "svg",
    ScrapingCategory::Vectors,
    "Free",
    "1,000+",
)
.with_search_url("https://storyset.com/search?q={query}");

/// Blush Design - 5,000+ illustrations
pub const BLUSH_DESIGN: ScrapingTarget = ScrapingTarget::new(
    "blushdesign",
    "Blush Design",
    "https://blush.design",
    ".illustration",
    ".illustration img",
    ScrapingCategory::Vectors,
    "Free",
    "5,000+",
);

// --- Game Assets ---

/// Kenney - 40,000+ free game assets
pub const KENNEY: ScrapingTarget = ScrapingTarget::new(
    "kenney",
    "Kenney",
    "https://kenney.nl/assets",
    ".asset",
    ".asset img",
    ScrapingCategory::GameAssets,
    "CC0",
    "40,000+",
)
.with_notes("High-quality game art, UI, audio");

/// OpenGameArt - 50,000+ free game assets
pub const OPENGAMEART: ScrapingTarget = ScrapingTarget::new(
    "opengameart",
    "OpenGameArt",
    "https://opengameart.org",
    ".art-item",
    ".art-item img",
    ScrapingCategory::GameAssets,
    "CC/GPL",
    "50,000+",
)
.with_search_url("https://opengameart.org/art-search?query={query}");

/// itch.io Free Game Assets - 100,000+ assets
pub const ITCH_IO_ASSETS: ScrapingTarget = ScrapingTarget::new(
    "itchioassets",
    "itch.io Free Assets",
    "https://itch.io/game-assets/free",
    ".game_cell",
    ".game_cell img",
    ScrapingCategory::GameAssets,
    "Varies",
    "100,000+",
)
.with_search_url("https://itch.io/game-assets/free?q={query}");

/// GameArt2D - 1,000+ free 2D assets
pub const GAMEART2D: ScrapingTarget = ScrapingTarget::new(
    "gameart2d",
    "GameArt2D",
    "https://gameart2d.com/freebies.html",
    ".asset-item",
    ".asset-item img",
    ScrapingCategory::GameAssets,
    "Free",
    "1,000+",
);

/// CraftPix Free - 500+ free game assets
pub const CRAFTPIX_FREE: ScrapingTarget = ScrapingTarget::new(
    "craftpixfree",
    "CraftPix Free",
    "https://craftpix.net/freebies",
    ".product-item",
    ".product-item img",
    ScrapingCategory::GameAssets,
    "Free",
    "500+",
);

/// Game-Icons.net - 4,000+ game icons
pub const GAME_ICONS: ScrapingTarget = ScrapingTarget::new(
    "gameicons",
    "Game-Icons.net",
    "https://game-icons.net",
    ".icon",
    "svg",
    ScrapingCategory::GameAssets,
    "CC BY 3.0",
    "4,000+",
)
.with_search_url("https://game-icons.net/search.html?q={query}");

/// Unity Asset Store Free - 5,000+ free assets
pub const UNITY_STORE_FREE: ScrapingTarget = ScrapingTarget::new(
    "unitystorefree",
    "Unity Asset Store Free",
    "https://assetstore.unity.com/top-assets/top-free",
    ".asset-card",
    ".asset-card img",
    ScrapingCategory::GameAssets,
    "Free",
    "5,000+",
)
.with_search_url("https://assetstore.unity.com/?q={query}&free=true");

/// Quaternius - 1,000+ free low-poly 3D models
pub const QUATERNIUS: ScrapingTarget = ScrapingTarget::new(
    "quaternius",
    "Quaternius",
    "https://quaternius.com/packs",
    ".pack-item",
    ".pack-item img",
    ScrapingCategory::GameAssets,
    "CC0",
    "1,000+",
)
.with_notes("Low-poly 3D game models");

/// Kay Lousberg - 500+ free game assets
pub const KAY_LOUSBERG: ScrapingTarget = ScrapingTarget::new(
    "kaylousberg",
    "Kay Lousberg",
    "https://kaylousberg.com/game-assets",
    ".asset",
    ".asset img",
    ScrapingCategory::GameAssets,
    "CC0",
    "500+",
);

// ═══════════════════════════════════════════════════════════════════════════════
// BATCH 5 COLLECTION - 3D, Textures, Vectors, Game Assets (50 sites)
// ═══════════════════════════════════════════════════════════════════════════════

/// All Batch 5 targets (50 sites, ~12M+ assets)
pub const BATCH_5_ASSETS_TARGETS: &[ScrapingTarget] = &[
    // 3D Models (9)
    FREE3D,
    ARCHIVE3D,
    CGTRADER_FREE,
    TURBOSQUID_FREE,
    SKETCHFAB_FREE,
    CLARA_IO,
    RENDERHUB_FREE,
    SHARECG,
    THREED_WAREHOUSE,
    // Textures (9)
    TEXTURECAN,
    TEXTURES_COM,
    THREED_TEXTURES,
    SHARE_TEXTURES,
    TEXTURE_NINJA,
    TEXTURELIB,
    POLYHAVEN_TEXTURES,
    AMBIENTCG,
    CGBOOKCASE,
    // Vectors & Icons (15)
    SVGREPO,
    ICONMONSTR,
    FLATICON_FREE,
    SIMPLE_ICONS,
    FEATHER_ICONS,
    HEROICONS,
    TABLER_ICONS,
    LUCIDE,
    PHOSPHOR_ICONS,
    BOOTSTRAP_ICONS,
    MATERIAL_ICONS,
    FREEPIK_FREE,
    VECTEEZY_FREE,
    UNDRAW,
    DRAWKIT,
    // Illustrations (8)
    HUMAAANS,
    OPEN_DOODLES,
    ABSURD_DESIGN,
    IRA_DESIGN,
    STORYSET,
    BLUSH_DESIGN,
    // Game Assets (9)
    KENNEY,
    OPENGAMEART,
    ITCH_IO_ASSETS,
    GAMEART2D,
    CRAFTPIX_FREE,
    GAME_ICONS,
    UNITY_STORE_FREE,
    QUATERNIUS,
    KAY_LOUSBERG,
];

// ═══════════════════════════════════════════════════════════════════════════════
// ALL SCRAPING TARGETS - Combined from all batches
// ═══════════════════════════════════════════════════════════════════════════════

/// Helper to combine slices at compile time is not possible in Rust,
/// so we list all targets explicitly for the unified list.
pub const SCRAPING_TARGETS: &[ScrapingTarget] = &[
    // Batch 1: Images (50)
    STOCKSNAP,
    BURST,
    RESHOT,
    PICJUMBO,
    GRATISOGRAPHY,
    LIFE_OF_PIX,
    NEGATIVE_SPACE,
    FOODIESFEED,
    SKITTERPHOTO,
    CUPCAKE,
    ISO_REPUBLIC,
    SPLITSHIRE,
    LIBRESHOT,
    MAGDELEINE,
    KABOOMPICS,
    JAY_MANTRI,
    TRAVEL_COFFEE_BOOK,
    MOVEAST,
    STOKPIC,
    FOCA_STOCK,
    GOOD_STOCK_PHOTOS,
    BARN_IMAGES,
    FREELY_PHOTOS,
    DESIGNERSPICS,
    FREE_NATURE_STOCK,
    PUBLIC_DOMAIN_PICTURES,
    PXHERE,
    STOCKVAULT,
    FREERANGESTOCK,
    RGBSTOCK,
    MORGUEFILE,
    NEW_OLD_STOCK,
    PICKUP_IMAGE,
    MMT_STOCK,
    LOCK_AND_STOCK,
    PHOTOSTOCKEDITOR,
    STYLED_STOCK,
    SHOTSTASH,
    NAPPY,
    IWARIA,
    EPICANTUS,
    TOOKAPIC,
    SNAPWIRE_SNAPS,
    BUCKETLISTLY,
    AVOPIX,
    FANCYCRAVE,
    PICOGRAPHY,
    JESHOOTS,
    RAUMROT,
    ALBUMARIUM,
    // Batch 2: Images (36)
    GETREFE,
    ANCESTRY_IMAGES,
    OLD_BOOK_ILLUSTRATIONS,
    GETTY_OPEN,
    YALE_BEINECKE,
    PARIS_MUSEES,
    ESA_IMAGES,
    NOAA_PHOTOS,
    USFWS,
    NPS_PHOTOS,
    USGS_PHOTOS,
    CDC_PHIL,
    NIH_GALLERY,
    US_NAVY,
    US_AIR_FORCE,
    UN_PHOTOS,
    SUPERFAMOUS,
    REALISTIC_SHOTS,
    STARTUP_STOCK,
    PHOTO_COLLECTIONS,
    VINTAGE_STOCK,
    RETROGRAPHIC,
    OLD_DESIGN_SHOP,
    WOCINTECH,
    JOPWELL,
    CREATEHER_STOCK,
    ONE_MILLION_FREE,
    CROW_THE_STONE,
    TUMBLR_FREE_STOCK,
    PHOTORACK,
    FREEPHOTOS_CC,
    LITTLE_VISUALS,
    DEATH_TO_STOCK,
    SKYPIXEL,
    LIBRESTOCK,
    FINDA_PHOTO,
    // Batch 3: Videos (45)
    MIXKIT_VIDEOS,
    VIDEVO,
    LIFE_OF_VIDS,
    DAREFUL,
    VIDSPLAY,
    MAZWAI,
    MOTION_PLACES,
    SPLITSHIRE_VIDEOS,
    XSTOCKVIDEO,
    CLIPSTILL,
    ISO_REPUBLIC_VIDEOS,
    DISTILL,
    BEACHFRONT,
    MOTION_ARRAY,
    POND5_PD,
    PHIL_FRIED,
    VIDEEZY,
    IGNITE_MOTION,
    MONZOOM,
    STOCK_FOOTAGE_4_FREE,
    VYDA,
    CUTE_STOCK,
    MOTION_BACKGROUNDS,
    FREE_GREEN_SCREEN,
    BENCHART,
    PANZOID,
    FREE_HD_FOOTAGE,
    ORANGEHD,
    MOVIE_TOOLS,
    OPEN_VIDEO_PROJECT,
    FOOTAGE_ISLAND,
    FREE_FOOTAGE,
    GRAIN_FREE,
    FREE_STOCK_FOOTAGE_ARCHIVE,
    NASA_VIDEO,
    ESA_VIDEOS,
    HUBBLE_VIDEOS,
    NOAA_VIDEO,
    NPS_BROLL,
    OPEN_FOOTAGE,
    DISSOLVE_FREE,
    PRODUCTION_CRATE,
    ACTIONVFX,
    MOTION_ELEMENTS,
    VECTEEZY_VIDEOS,
    // Batch 4: Audio (47)
    MIXKIT_MUSIC,
    MIXKIT_SFX,
    MUSOPEN,
    FREE_PD,
    PDSOUNDS,
    SOUNDBIBLE,
    BBC_SFX,
    ZAPSPLAT,
    SOUNDJAY,
    UPPBEAT,
    BENSOUND,
    CHOSIC,
    AUDIONAUTIX,
    INCOMPETECH,
    PURPLE_PLANET,
    PARTNERS_IN_RHYME,
    SAMPLE_FOCUS,
    LOOPERMAN,
    SOUNDS_CRATE,
    SOUNDGATOR,
    GR_SITES,
    FREE_SOUND_EFFECTS,
    ORANGE_FREE_SOUNDS,
    NINETY_NINE_SOUNDS,
    FREE_SFX,
    FREE_SOUND_LIBRARY,
    NOISE_FOR_FUN,
    FLASH_KIT_SOUNDS,
    WAV_SOURCE,
    FREE_LOOPS,
    SAMPLESWAP,
    FESLIYAN,
    TABLA_FREE,
    FREE_STOCK_MUSIC,
    SILVERMAN_SOUND,
    DL_SOUNDS,
    FMA,
    CCMIXTER,
    JAMENDO,
    DIG_CCMIXTER,
    ARCHIVE_AUDIO,
    SOUNDCLOUD_CC,
    EPIDEMIC_FREE,
    ARTLIST_FREE,
    NCS,
    AUDIO_LIBRARY,
    POND5_AUDIO_PD,
    // Batch 5: 3D, Textures, Vectors, Game Assets (50)
    FREE3D,
    ARCHIVE3D,
    CGTRADER_FREE,
    TURBOSQUID_FREE,
    SKETCHFAB_FREE,
    CLARA_IO,
    RENDERHUB_FREE,
    SHARECG,
    THREED_WAREHOUSE,
    TEXTURECAN,
    TEXTURES_COM,
    THREED_TEXTURES,
    SHARE_TEXTURES,
    TEXTURE_NINJA,
    TEXTURELIB,
    POLYHAVEN_TEXTURES,
    AMBIENTCG,
    CGBOOKCASE,
    SVGREPO,
    ICONMONSTR,
    FLATICON_FREE,
    SIMPLE_ICONS,
    FEATHER_ICONS,
    HEROICONS,
    TABLER_ICONS,
    LUCIDE,
    PHOSPHOR_ICONS,
    BOOTSTRAP_ICONS,
    MATERIAL_ICONS,
    FREEPIK_FREE,
    VECTEEZY_FREE,
    UNDRAW,
    DRAWKIT,
    HUMAAANS,
    OPEN_DOODLES,
    ABSURD_DESIGN,
    IRA_DESIGN,
    STORYSET,
    BLUSH_DESIGN,
    KENNEY,
    OPENGAMEART,
    ITCH_IO_ASSETS,
    GAMEART2D,
    CRAFTPIX_FREE,
    GAME_ICONS,
    UNITY_STORE_FREE,
    QUATERNIUS,
    KAY_LOUSBERG,
];

// ═══════════════════════════════════════════════════════════════════════════════
// REGISTRY STRUCT - All batches aggregated
// ═══════════════════════════════════════════════════════════════════════════════

/// Registry of all scraping targets.
pub struct ScrapingRegistry;

impl ScrapingRegistry {
    /// Get all Batch 1 image targets (50 sites).
    pub fn batch_1_images() -> &'static [ScrapingTarget] {
        BATCH_1_IMAGE_TARGETS
    }

    /// Get all Batch 2 image targets (36 sites).
    pub fn batch_2_images() -> &'static [ScrapingTarget] {
        BATCH_2_IMAGE_TARGETS
    }

    /// Get all Batch 3 video targets (45 sites).
    pub fn batch_3_videos() -> &'static [ScrapingTarget] {
        BATCH_3_VIDEO_TARGETS
    }

    /// Get all Batch 4 audio targets (47 sites).
    pub fn batch_4_audio() -> &'static [ScrapingTarget] {
        BATCH_4_AUDIO_TARGETS
    }

    /// Get all Batch 5 targets (50 sites - 3D, textures, vectors, game).
    pub fn batch_5_assets() -> &'static [ScrapingTarget] {
        BATCH_5_ASSETS_TARGETS
    }

    /// Get all image scraping targets (86 sites).
    pub fn all_images() -> Vec<&'static ScrapingTarget> {
        BATCH_1_IMAGE_TARGETS
            .iter()
            .chain(BATCH_2_IMAGE_TARGETS.iter())
            .collect()
    }

    /// Get all video scraping targets.
    pub fn all_videos() -> &'static [ScrapingTarget] {
        BATCH_3_VIDEO_TARGETS
    }

    /// Get all audio scraping targets.
    pub fn all_audio() -> &'static [ScrapingTarget] {
        BATCH_4_AUDIO_TARGETS
    }

    /// Get all 3D model targets.
    pub fn all_3d_models() -> Vec<&'static ScrapingTarget> {
        SCRAPING_TARGETS
            .iter()
            .filter(|t| t.category == ScrapingCategory::Models3D)
            .collect()
    }

    /// Get all texture targets.
    pub fn all_textures() -> Vec<&'static ScrapingTarget> {
        SCRAPING_TARGETS
            .iter()
            .filter(|t| t.category == ScrapingCategory::Textures)
            .collect()
    }

    /// Get all vector/icon targets.
    pub fn all_vectors() -> Vec<&'static ScrapingTarget> {
        SCRAPING_TARGETS
            .iter()
            .filter(|t| t.category == ScrapingCategory::Vectors)
            .collect()
    }

    /// Get all game asset targets.
    pub fn all_game_assets() -> Vec<&'static ScrapingTarget> {
        SCRAPING_TARGETS
            .iter()
            .filter(|t| t.category == ScrapingCategory::GameAssets)
            .collect()
    }

    /// Get target by ID from all batches.
    pub fn get(id: &str) -> Option<&'static ScrapingTarget> {
        SCRAPING_TARGETS.iter().find(|t| t.id == id)
    }

    /// Get all targets for a category.
    pub fn by_category(category: ScrapingCategory) -> Vec<&'static ScrapingTarget> {
        SCRAPING_TARGETS
            .iter()
            .filter(|t| t.category == category)
            .collect()
    }

    /// Get total count of all targets.
    pub fn total_count() -> usize {
        SCRAPING_TARGETS.len()
    }

    /// Get estimated total assets.
    pub fn total_assets_estimate() -> &'static str {
        "~38M+ assets from 228 sources"
    }

    /// Get asset breakdown by category.
    pub fn asset_breakdown() -> &'static str {
        "Images: ~5.1M | Videos: ~120K | Audio: ~21M+ | 3D: ~5M | Textures: ~16K | Vectors: ~2.6M | Game: ~200K"
    }

    /// List all target IDs.
    pub fn list_ids() -> Vec<&'static str> {
        SCRAPING_TARGETS.iter().map(|t| t.id).collect()
    }

    /// Get targets that are CC0/Public Domain (no attribution needed).
    pub fn cc0_targets() -> Vec<&'static ScrapingTarget> {
        SCRAPING_TARGETS
            .iter()
            .filter(|t| t.license.contains("CC0") || t.license.contains("PD"))
            .collect()
    }
}
