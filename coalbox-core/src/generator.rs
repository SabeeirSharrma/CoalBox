use rand::Rng;
use serde::{Deserialize, Serialize};

const UPPERCASE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWERCASE: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const DIGITS: &[u8] = b"0123456789";
const SYMBOLS: &[u8] = b"!@#$%^&*()_+-=[]{}|;:,.<>?";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordConfig {
    pub length: usize,
    pub uppercase: bool,
    pub lowercase: bool,
    pub numbers: bool,
    pub symbols: bool,
    pub custom_symbols: Option<String>,
    pub exclude_chars: Option<String>,
}

impl Default for PasswordConfig {
    fn default() -> Self {
        Self {
            length: 20,
            uppercase: true,
            lowercase: true,
            numbers: true,
            symbols: true,
            custom_symbols: None,
            exclude_chars: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassphraseConfig {
    pub word_count: usize,
    pub separator: String,
    pub capitalize: bool,
    pub include_number: bool,
}

impl Default for PassphraseConfig {
    fn default() -> Self {
        Self {
            word_count: 6,
            separator: " ".to_string(),
            capitalize: true,
            include_number: false,
        }
    }
}

pub fn generate_password(config: &PasswordConfig) -> String {
    let mut charset = Vec::new();

    if config.uppercase {
        charset.extend_from_slice(UPPERCASE);
    }
    if config.lowercase {
        charset.extend_from_slice(LOWERCASE);
    }
    if config.numbers {
        charset.extend_from_slice(DIGITS);
    }

    if let Some(ref custom) = config.custom_symbols {
        charset.extend_from_slice(custom.as_bytes());
    } else if config.symbols {
        charset.extend_from_slice(SYMBOLS);
    }

    if let Some(ref exclude) = config.exclude_chars {
        charset.retain(|c| !exclude.contains(*c as char));
    }

    if charset.is_empty() {
        return String::new();
    }

    let mut rng = rand::thread_rng();
    (0..config.length)
        .map(|_| {
            let idx = rng.gen_range(0..charset.len());
            charset[idx] as char
        })
        .collect()
}

pub fn generate_passphrase(config: &PassphraseConfig) -> String {
    let words = EFF_WORDS;
    let word_count = config.word_count.min(words.len());
    let mut rng = rand::thread_rng();

    let selected: Vec<String> = (0..word_count)
        .map(|_| {
            let idx = rng.gen_range(0..words.len());
            let word = words[idx];
            if config.capitalize {
                let mut chars = word.chars();
                match chars.next() {
                    Some(first) => {
                        let rest: String = chars.collect();
                        format!("{}{}", first.to_uppercase(), rest)
                    }
                    None => String::new(),
                }
            } else {
                word.to_string()
            }
        })
        .collect();

    let mut result = selected.join(&config.separator);

    if config.include_number {
        let num = rng.gen_range(0..100);
        result.push_str(&config.separator);
        result.push_str(&num.to_string());
    }

    result
}

const EFF_WORDS: &[&str] = &[
    "abacus", "abdomen", "abdominal", "abide", "abiding", "ability", "ablaze",
    "able", "abnormal", "abrasion", "abrasive", "abreast", "abridge", "abroad",
    "abruptly", "absence", "absentee", "absently", "absinthe", "absolute", "absolve",
    "abstain", "abstract", "absurd", "accent", "acclaim", "acclimate", "accompany",
    "account", "accuracy", "accurate", "accustom", "acetone", "achiness", "aching",
    "acid", "acorn", "acquaint", "acquire", "acre", "acrobat", "acronym", "acting",
    "action", "activate", "activator", "active", "activism", "activist", "activity",
    "actress", "acts", "acutely", "acuteness", "aeration", "aerobics", "aerosol",
    "aerospace", "afar", "affair", "affected", "affecting", "affection", "affidavit",
    "affiliate", "affirm", "affix", "afflicted", "affluent", "afford", "affront",
    "aflame", "afloat", "aflutter", "afoot", "afraid", "afterglow", "afterlife",
    "aftermath", "aftermost", "afternoon", "aged", "ageless", "agency", "agenda",
    "agent", "aggregate", "aghast", "agile", "agility", "aging", "agnostic", "agonize",
    "agonizing", "agony", "agreeable", "agreeably", "agreed", "agreeing", "agreement",
    "aground", "ahead", "ahoy", "aide", "aids", "aim", "ajar", "alabaster", "alarm",
    "albatross", "album", "alfalfa", "algebra", "algorithm", "alias", "alibi",
    "alienable", "alienate", "aliens", "alike", "alive", "alkaline", "alkalize",
    "almanac", "almighty", "almost", "aloe", "aloft", "aloha", "alone", "alongside",
    "aloof", "alphabet", "alright", "although", "altitude", "alto", "aluminum",
    "alumni", "always", "amaretto", "amaze", "amazingly", "amber", "ambiance",
    "ambiguity", "ambiguous", "ambition", "ambitious", "ambulance", "ambush",
    "amendable", "amendment", "amends", "amenity", "amiable", "amicably", "amid",
    "amigo", "amino", "amiss", "ammonia", "ammonium", "amnesty", "amniotic", "among",
    "amount", "amperage", "ample", "amplifier", "amplify", "amply", "amuck", "amulet",
    "amusable", "amused", "amusement", "amuser", "amusing", "anaconda", "anaerobic",
    "anagram", "anatomist", "anatomy", "anchor", "anchovy", "ancient", "android",
    "anemia", "anemic", "aneurism", "anew", "angelfish", "angelic", "anger", "angled",
    "angler", "angles", "angling", "angrily", "angriness", "anguished", "angular",
    "animal", "animate", "animating", "animation", "animator", "anime", "animosity",
    "ankle", "annex", "annotate", "announcer", "annoying", "annually", "annuity",
    "anointer", "another", "answering", "antacid", "antarctic", "anteater", "antelope",
    "antennae", "anthem", "anthill", "anthology", "antibody", "antics", "antidote",
    "antihero", "antiquely", "antiques", "antiquity", "antirust", "antitoxic",
    "antitrust", "antiviral", "antivirus", "antler", "antonym", "antsy", "anvil",
    "anybody", "anyhow", "anymore", "anyone", "anyplace", "anything", "anytime",
    "anyway", "anywhere", "aorta", "apache", "apostle", "appealing", "appear",
    "appease", "appeasing", "appendage", "appendix", "appetite", "appetizer",
    "applaud", "applause", "apple", "appliance", "applicant", "applied", "apply",
    "appointee", "appraisal", "appraiser", "apprehend", "approach", "approval",
    "approve", "apricot", "april", "apron", "aptitude", "aptly", "aqua", "aqueduct",
    "arbitrary", "arbitrate", "ardently", "area", "arena", "arguable", "arguably",
    "argue", "arise", "armadillo", "armband", "armchair", "armed", "armful", "armhole",
    "arming", "armless", "armoire", "armored", "armory", "armrest", "army", "aroma",
    "arose", "around", "arousal", "arrange", "array", "arrest", "arrival", "arrive",
    "arrogance", "arrogant", "arson", "art", "ascend", "ascension", "ascent",
    "ascertain", "ashamed", "ashen", "ashes", "ashy", "aside", "askew", "asleep",
    "asparagus", "aspect", "aspirate", "aspire", "aspirin", "astonish", "astound",
    "astride", "astrology", "astronaut", "astronomy", "astute", "atlantic", "atlas",
    "atom", "atonable", "atop", "atrium", "atrocious", "atrophy", "attach", "attain",
    "attempt", "attendant", "attendee", "attention", "attentive", "attest", "attic",
    "attire", "attitude", "attractor", "attribute", "atypical", "auction", "audacious",
    "audacity", "audible", "audibly", "audience", "audio", "audition", "augmented",
    "august", "authentic", "author", "autism", "autistic", "autograph", "automaker",
    "automated", "automatic", "autopilot", "available", "avalanche", "avatar", "avenge",
    "avenging", "avenue", "average", "aversion", "avert", "aviation", "aviator", "avid",
    "avoid", "await", "awaken", "award", "aware", "awhile", "awkward", "awning",
    "awoke", "awry", "axis", "babble", "babbling", "babied", "baboon", "backache",
    "backboard", "backboned", "backdrop", "backed", "backer", "backfield", "backfire",
    "backhand", "backing", "backlands", "backlash", "backless", "backlight", "backlit",
    "backlog", "backpack", "backpedal", "backrest", "backroom", "backshift", "backside",
    "backslid", "backspace", "backspin", "backstab", "backstage", "backtalk", "backtrack",
    "backup", "backward", "backwash", "backwater", "backyard", "bacon", "bacteria",
    "bacterium", "badass", "badge", "badland", "badly", "badness", "baffle", "baffling",
    "bagel", "bagful", "baggage", "bagged", "baggie", "bagginess", "bagging", "baggy",
    "bagpipe", "baguette", "baked", "bakery", "bakeshop", "baking", "balance",
    "balancing", "balcony", "balmy", "balsamic", "bamboo", "banana", "banish",
    "banister", "banjo", "bankable", "bankbook", "banked", "banker", "banking",
    "banknote", "bankroll", "banner", "bannister", "banshee", "banter", "barbecue",
    "barbed", "barbell", "barber", "barcode", "barge", "bargraph", "barista",
    "baritone", "barley", "barmaid", "barman", "barn", "barometer", "barrack",
    "barracuda", "barrel", "barrette", "barricade", "barrier", "barstool", "bartender",
    "barterer", "bash", "basically", "basics", "basil", "basin", "basis", "basket",
    "batboy", "batch", "bath", "baton", "bats", "battalion", "battered", "battering",
    "battery", "batting", "battle", "bauble", "bazooka", "blabber", "bladder", "blade",
    "blah", "blame", "blaming", "blanching", "blandness", "blank", "blaspheme",
    "blasphemy", "blast", "blatancy", "blatantly", "blazer", "blazing", "bleach",
    "bleak", "bleep", "blemish", "blend", "bless", "blighted", "blimp", "bling",
    "blinked", "blinker", "blinking", "blinks", "blip", "blissful", "blitz", "blizzard",
    "bloated", "bloating", "blob", "blog", "bloomers", "blooming", "blooper", "blot",
    "blouse", "blubber", "bluff", "bluish", "blunderer", "blunt", "blurb", "blurred",
    "blurry", "blurt", "blush", "blustery", "boaster", "boastful", "boasting", "boat",
    "bobbed", "bobbing", "bobble", "bobcat", "bobsled", "bobtail", "bodacious", "body",
    "bogged", "boggle", "bogus", "boil", "bok", "bolster", "bolt", "bonanza", "bonded",
    "bonding", "bondless", "boned", "bonehead", "boneless", "bonelike", "boney",
    "bonfire", "bonnet", "bonsai", "bonus", "bony", "boogeyman", "boogieman", "book",
    "boondocks", "booted", "booth", "bootie", "booting", "bootlace", "bootleg", "boots",
    "boozy", "borax", "boring", "borough", "borrower", "borrowing", "boss", "botanical",
    "botanist", "botany", "botch", "both", "bottle", "bottling", "bottom", "bounce",
    "bouncing", "bouncy", "bounding", "boundless", "bountiful", "bovine", "boxcar",
    "boxer", "boxing", "boxlike", "boxy", "breach", "breath", "breeches", "breeching",
    "breeder", "breeding", "breeze", "breezy", "brethren", "brewery", "brewing", "briar",
    "bribe", "brick", "bride", "bridged", "brigade", "bright", "brilliant", "brim",
    "bring", "brink", "brisket", "briskly", "briskness", "bristle", "brittle",
    "broadband", "broadcast", "broaden", "broadly", "broadness", "broadside", "broadways",
    "broiler", "broiling", "broken", "broker", "bronchial", "bronco", "bronze",
    "bronzing", "brook", "broom", "brought", "browbeat", "brownnose", "browse",
    "browsing", "bruising", "brunch", "brunette", "brunt", "brush", "brussels", "brute",
    "brutishly", "bubble", "bubbling", "bubbly", "buccaneer", "bucked", "bucket",
    "buckle", "buckshot", "buckskin", "bucktooth", "buckwheat", "buddhism", "buddhist",
    "budding", "buddy", "budget", "buffalo", "buffed", "buffer", "buffing", "buffoon",
    "buggy", "bulb", "bulge", "bulginess", "bulgur", "bulk", "bulldog", "bulldozer",
    "bullfight", "bullfrog", "bullhorn", "bullion", "bullish", "bullpen", "bullring",
    "bullseye", "bullwhip", "bully", "bunch", "bundle", "bungee", "bunion", "bunkbed",
    "bunkhouse", "bunkmate", "bunny", "bunt", "busboy", "bush", "busily", "busload",
    "bust", "busybody", "buzz", "cabana", "cabbage", "cabbie", "cabdriver", "cable",
    "caboose", "cache", "cackle", "cacti", "cactus", "caddie", "caddy", "cadet",
    "cadillac", "cadmium", "cage", "cahoots", "cake", "calamari", "calamity", "calcium",
    "calculate", "calculus", "caliber", "calibrate", "calm", "caloric", "calorie",
    "calzone", "camcorder", "cameo", "camera", "camisole", "camper", "campfire",
    "camping", "campsite", "campus", "canal", "canary", "cancel", "candied", "candle",
    "candy", "cane", "canine", "canister", "cannabis", "canned", "canning", "cannon",
    "cannot", "canola", "canon", "canopener", "canopy", "canteen", "canyon", "capable",
    "capably", "capacity", "cape", "capillary", "capital", "capitol", "capped",
    "capricorn", "capsize", "capsule", "caption", "captivate", "captive", "captivity",
    "capture", "caramel", "carat", "caravan", "carbon", "cardboard", "carded", "cardiac",
    "cardigan", "cardinal", "cardstock", "carefully", "caregiver", "careless", "caress",
    "caretaker", "cargo", "caring", "carless", "carload", "carmaker", "carnage",
    "carnation", "carnival", "carnivore", "carol", "carpenter", "carpentry", "carpool",
    "carport", "carried", "carrot", "carrousel", "carry", "cartel", "cartload", "carton",
    "cartoon", "cartridge", "cartwheel", "carve", "carving", "carwash", "cascade", "case",
    "cash", "casing", "casino", "casket", "cassette", "casually", "casualty", "catacomb",
    "catalog", "catalyst", "catalyze", "catapult", "cataract", "catatonic", "catcall",
    "catchable", "catcher", "catching", "catchy", "caterer", "catering", "catfight",
    "catfish", "cathedral", "cathouse", "catlike", "catnap", "catnip", "catsup", "cattail",
    "cattishly", "cattle", "catty", "catwalk", "caucasian", "caucus", "causal",
    "causation", "cause", "causing", "cauterize", "caution", "cautious", "cavalier",
    "cavalry", "caviar", "cavity", "cedar", "celery", "celestial", "celibacy",
    "celibate", "celtic", "cement", "census", "ceramics", "ceremony", "certainly",
    "certainty", "certified", "certify", "cesarean", "cesspool", "chafe", "chaffing",
    "chain", "chair", "chalice", "challenge", "chamber", "chamomile", "champion", "chance",
    "change", "channel", "chant", "chaos", "chaperone", "chaplain", "chapped", "chaps",
    "chapter", "character", "charbroil", "charcoal", "charger", "charging", "chariot",
    "charity", "charm", "charred", "charter", "charting", "chase", "chasing", "chaste",
    "chastise", "chastity", "chatroom", "chatter", "chatting", "chatty", "cheating",
    "cheddar", "cheek", "cheer", "cheese", "cheesy", "chef", "chemicals", "chemist",
    "chemo", "cherisher", "cherub", "chess", "chest", "chevron", "chevy", "chewable",
    "chewer", "chewing", "chewy", "chief", "chihuahua", "childcare", "childhood",
    "childish", "childless", "childlike", "chili", "chill", "chimp", "chip", "chirping",
    "chirpy", "chitchat", "chivalry", "chive", "chloride", "chlorine", "choice",
    "chokehold", "choking", "chomp", "chooser", "choosing", "choosy", "chop", "chosen",
    "chowder", "chowtime", "chrome", "chubby", "chuck", "chug", "chummy", "chump",
    "chunk", "churn", "chute", "cider", "cilantro", "cinch", "cinema", "cinnamon",
    "circle", "circling", "circular", "circulate", "circus", "citable", "citadel",
    "citation", "citizen", "citric", "citrus", "city", "civic", "civil", "clad",
    "claim", "clambake", "clammy", "clamor", "clamp", "clamshell", "clang", "clanking",
    "clapped", "clapper", "clapping", "clarify", "clarinet", "clarity", "clash", "clasp",
    "class", "clatter", "clause", "clavicle", "claw", "clay", "clean", "clear", "cleat",
    "cleaver", "cleft", "clench", "clergyman", "clerical", "clerk", "clever", "clicker",
    "client", "climate", "climatic", "cling", "clinic", "clinking", "clip", "clique",
    "cloak", "clobber", "clock", "clone", "cloning", "closable", "closure", "clothes",
    "clothing", "cloud", "clover", "clubbed", "clubbing", "clubhouse", "clump", "clumsily",
    "clumsy", "clunky", "clustered", "clutch", "clutter", "coach", "coagulant", "coastal",
    "coaster", "coasting", "coastland", "coastline", "coat", "coauthor", "cobalt",
    "cobbler", "cobweb", "cocoa", "coconut", "cod", "coeditor", "coerce", "coexist",
    "coffee", "cofounder", "cognition", "cognitive", "cogwheel", "coherence", "coherent",
    "cohesive", "coil", "coke", "cola", "cold", "coleslaw", "coliseum", "collage",
    "collapse", "collar", "collected", "collector", "collide", "collie", "collision",
    "colonial", "colonist", "colonize", "colony", "colossal", "colt", "coma", "come",
    "comfort", "comfy", "comic", "coming", "comma", "commence", "commend", "comment",
    "commerce", "commode", "commodity", "commodore", "common", "commotion", "commute",
    "commuting", "compacted", "compacter", "compactly", "compactor", "companion", "company",
    "compare", "compel", "compile", "comply", "component", "composed", "composer",
    "composite", "compost", "composure", "compound", "compress", "comprised", "computer",
    "computing", "comrade", "concave", "conceal", "conceded", "concept", "concerned",
    "concert", "conch", "concierge", "concise", "conclude", "concrete", "concur",
    "condense", "condiment", "condition", "condone", "conducive", "conductor", "conduit",
    "cone", "confess", "confetti", "confidant", "confident", "confider", "confiding",
    "configure", "confined", "confining", "confirm", "conflict", "conform", "confound",
    "confront", "confused", "confusing", "confusion", "congenial", "congested", "congrats",
    "congress", "conical", "conjoined", "conjure", "conjuror", "connected", "connector",
    "consensus", "consent", "console", "consoling", "consonant", "constable", "constant",
    "constrain", "constrict", "construct", "consult", "consumer", "consuming", "contact",
    "container", "contempt", "contend", "contented", "contently", "contents", "contest",
    "context", "contort", "contour", "contrite", "control", "contusion", "convene",
    "convent", "copartner", "cope", "copied", "copier", "copilot", "coping", "copious",
    "copper", "copy", "coral", "cork", "cornball", "cornbread", "corncob", "cornea",
    "corned", "corner", "cornfield", "cornflake", "cornhusk", "cornmeal", "cornstalk",
    "corny", "coronary", "coroner", "corporal", "corporate", "corral", "correct",
    "corridor", "corrode", "corroding", "corrosive", "corsage", "corset", "cortex",
    "cosigner", "cosmetics", "cosmic", "cosmos", "cosponsor", "cost", "cottage", "cotton",
    "couch", "cough", "could", "countable", "countdown", "counting", "countless", "country",
    "county", "courier", "covenant", "cover", "coveted", "coveting", "coyness", "cozily",
    "coziness", "cozy", "crabbing", "crabgrass", "crablike", "crabmeat", "cradle",
    "cradling", "crafter", "craftily", "craftsman", "craftwork", "crafty", "cramp",
    "cranberry", "crane", "cranial", "cranium", "crank", "crate", "crave", "craving",
    "crawfish", "crawlers", "crawling", "crayfish", "crayon", "crazed", "crazily",
    "craziness", "crazy", "creamed", "creamer", "creamlike", "crease", "creasing",
    "creatable", "create", "creation", "creative", "creature", "credible", "credibly",
    "credit", "creed", "creme", "creole", "crepe", "crept", "crescent", "crested",
    "cresting", "crestless", "crevice", "crewless", "crewman", "crewmate", "crib",
    "cricket", "cried", "crier", "crimp", "crimson", "cringe", "cringing", "crinkle",
    "crinkly", "crisped", "crisping", "crisply", "crispness", "crispy", "criteria",
    "critter", "croak", "crock", "crook", "croon", "crop", "cross", "crouch", "crouton",
    "crowbar", "crowd", "crown", "crucial", "crudely", "crudeness", "cruelly", "cruelness",
    "cruelty", "crumb", "crummiest", "crummy", "crumpet", "crumpled", "cruncher",
    "crunching", "crunchy", "crusader", "crushable", "crushed", "crusher", "crushing",
    "crust", "crux", "crying", "cryptic", "crystal", "cubbyhole", "cube", "cubical",
    "cubicle", "cucumber", "cuddle", "cuddly", "cufflink", "culinary", "culminate",
    "culpable", "culprit", "cultivate", "cultural", "culture", "cupbearer", "cupcake",
    "cupid", "cupped", "cupping", "curable", "curator", "curdle", "cure", "curfew",
    "curing", "curled", "curler", "curliness", "curling", "curly", "curry", "curse",
    "cursive", "cursor", "curtain", "curtly", "curtsy", "curvature", "curve", "curvy",
    "cushy", "cusp", "cussed", "custard", "custodian", "custody", "customary", "customer",
    "customize", "customs", "cut", "cycle", "cyclic", "cycling", "cyclist", "cylinder",
    "cymbal", "cytoplasm", "cytoplast", "dab", "dad", "daffodil", "dagger", "daily",
    "daintily", "dainty", "dairy", "daisy", "dallying", "dance", "dancing", "dandelion",
    "dander", "dandruff", "dandy", "danger", "dangle", "dangling", "daredevil", "dares",
    "daringly", "darkened", "darkening", "darkish", "darkness", "darkroom", "darling",
    "darn", "dart", "darwinism", "dash", "dastardly", "data", "datebook", "dating",
    "daughter", "daunting", "dawdler", "dawn", "daybed", "daybreak", "daycare", "daydream",
    "daylight", "daylong", "dayroom", "daytime", "dazzler", "dazzling", "deacon",
    "deafening", "deafness", "dealer", "dealing", "dealmaker", "dealt", "dean",
    "debatable", "debate", "debating", "debit", "debrief", "debtless", "debtor", "debug",
    "debunk", "decade", "decaf", "decal", "decathlon", "decay", "deceased", "deceit",
    "deceiver", "deceiving", "december", "decency", "decent", "deception", "deceptive",
    "decibel", "decidable", "decimal", "decimeter", "decipher", "deck", "declared",
    "decline", "decode", "decompose", "decorated", "decorator", "decoy", "decrease",
    "decree", "dedicate", "dedicator", "deduce", "deduct", "deed", "deem", "deepen",
    "deeply", "deepness", "deface", "defacing", "defame", "default", "defeat", "defection",
    "defective", "defendant", "defender", "defense", "defensive", "deferral", "deferred",
    "defiance", "defiant", "defile", "defiling", "define", "definite", "deflate",
    "deflation", "deflator", "deflected", "deflector", "defog", "deforest", "defraud",
    "defrost", "deftly", "defuse", "defy", "degraded", "degrading", "degrease", "degree",
    "dehydrate", "deity", "dejected", "delay", "delegate", "delegator", "delete",
    "deletion", "delicacy", "delicate", "delicious", "delighted", "delirious", "delirium",
    "deliverer", "delivery", "delouse", "delta", "deluge", "delusion", "deluxe",
    "demanding", "demeaning", "demeanor", "demise", "democracy", "democrat", "demote",
    "demotion", "demystify", "denatured", "deniable", "denial", "denim", "denote", "dense",
    "density", "dental", "dentist", "denture", "deny", "deodorant", "deodorize", "departed",
    "departure", "depict", "deplete", "depletion", "deplored", "deploy", "deport", "depose",
    "depraved", "depravity", "deprecate", "depress", "deprive", "depth", "deputize",
    "deputy", "derail", "deranged", "derby", "derived", "desecrate", "deserve",
    "deserving", "designate", "designed", "designer", "designing", "deskbound", "desktop",
    "deskwork", "desolate", "despair", "despise", "despite", "destiny", "destitute",
    "destruct", "detached", "detail", "detection", "detective", "detector", "detention",
    "detergent", "detest", "detonate", "detonator", "detoxify", "detract", "deuce",
    "devalue", "deviancy", "deviant", "deviate", "deviation", "deviator", "device",
    "devious", "devotedly", "devotee", "devotion", "devourer", "devouring", "devoutly",
    "dexterity", "dexterous", "diabetes", "diabetic", "diabolic", "diagnoses", "diagnosis",
    "diagram", "dial", "diameter", "diaper", "diaphragm", "diary", "dice", "dicing",
    "dictate", "dictation", "dictator", "difficult", "diffused", "diffuser", "diffusion",
    "diffusive", "dig", "dilation", "diligence", "diligent", "dill", "dilute", "dime",
    "diminish", "dimly", "dimmed", "dimmer", "dimness", "dimple", "diner", "dingbat",
    "dinghy", "dinginess", "dingo", "dingy", "dining", "dinner", "diocese", "dioxide",
    "diploma", "dipped", "dipper", "dipping", "directed", "direction", "directive",
    "directly", "directory", "direness", "dirtiness", "disabled", "disagree", "disallow",
    "disarm", "disarray", "disaster", "disband", "disbelief", "disburse", "discard",
    "discern", "discharge", "disclose", "discolor", "discount", "discourse", "discover",
    "discuss", "disdain", "disengage", "disfigure", "disgrace", "dish", "disinfect",
    "disjoin", "disk", "dislike", "disliking", "dislocate", "dislodge", "disloyal",
    "dismantle", "dismay", "dismiss", "dismount", "disobey", "disorder", "disown",
    "disparate", "disparity", "dispatch", "dispense", "dispersal", "dispersed",
    "disperser", "displace", "display", "displease", "disposal", "dispose", "disprove",
    "dispute", "disregard", "disrupt", "dissuade", "distance", "distant", "distaste",
    "distill", "distinct", "distort", "distract", "distress", "district", "distrust",
    "ditch", "ditto", "ditzy", "dividable", "divided", "dividend", "dividers", "dividing",
    "divinely", "diving", "divinity", "divisible", "divisibly", "division", "divisive",
    "divorcee", "dizziness", "dizzy", "doable", "docile", "dock", "doctrine", "document",
    "dodge", "dodgy", "doily", "doing", "dole", "dollar", "dollhouse", "dollop", "dolly",
    "dolphin", "domain", "domelike", "domestic", "dominion", "dominoes", "donated",
    "donation", "donator", "donor", "donut", "doodle", "doorbell", "doorframe", "doorknob",
    "doorman", "doormat", "doornail", "doorpost", "doorstep", "doorstop", "doorway",
    "doozy", "dork", "dormitory", "dorsal", "dosage", "dose", "dotted", "doubling",
    "douche", "dove", "down", "dowry", "doze", "drab", "dragging", "dragonfly",
    "dragonish", "dragster", "drainable", "drainage", "drained", "drainer", "drainpipe",
    "dramatic", "dramatize", "drank", "drapery", "drastic", "draw", "dreaded", "dreadful",
    "dreadlock", "dreamboat", "dreamily", "dreamland", "dreamless", "dreamlike", "dreamt",
    "dreamy", "drearily", "dreary", "drench", "dress", "drew", "dribble", "dried",
    "drier", "drift", "driller", "drilling", "drinkable", "drinking", "dripping", "drippy",
    "drivable", "driven",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_password_length() {
        let config = PasswordConfig::default();
        let password = generate_password(&config);
        assert_eq!(password.len(), 20);
    }

    #[test]
    fn test_generate_password_custom_length() {
        let config = PasswordConfig {
            length: 32,
            ..Default::default()
        };
        let password = generate_password(&config);
        assert_eq!(password.len(), 32);
    }

    #[test]
    fn test_generate_password_lowercase_only() {
        let config = PasswordConfig {
            uppercase: false,
            lowercase: true,
            numbers: false,
            symbols: false,
            ..Default::default()
        };
        let password = generate_password(&config);
        assert!(password.chars().all(|c| c.is_lowercase()));
    }

    #[test]
    fn test_generate_passphrase_word_count() {
        let config = PassphraseConfig::default();
        let passphrase = generate_passphrase(&config);
        let words: Vec<&str> = passphrase.split_whitespace().collect();
        assert_eq!(words.len(), 6);
    }

    #[test]
    fn test_generate_passphrase_capitalized() {
        let config = PassphraseConfig {
            capitalize: true,
            ..Default::default()
        };
        let passphrase = generate_passphrase(&config);
        for word in passphrase.split_whitespace() {
            let first = word.chars().next().unwrap();
            assert!(first.is_uppercase());
        }
    }

    #[test]
    fn test_generate_passphrase_with_number() {
        let config = PassphraseConfig {
            include_number: true,
            separator: "-".to_string(),
            ..Default::default()
        };
        let passphrase = generate_passphrase(&config);
        let parts: Vec<&str> = passphrase.split('-').collect();
        assert_eq!(parts.len(), 7);
        assert!(parts[6].parse::<i32>().is_ok());
    }
}
