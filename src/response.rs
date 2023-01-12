extern crate serde;
extern crate serde_derive;
extern crate serde_json;

use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt;
//use std::fmt::Display;
use std::str::FromStr;

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

enum Version {
    V1_0,
}

// /// use display to serialize a value
// fn use_display<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         T: Display,
//         S: Serializer
// {
//     serializer.collect_str(value)
// }


impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            Version::V1_0 => "1.0",
        };
        write!(f, "{}", s)
    }
}

impl Response {
    /// Constructs a new response with only required elements
    pub fn new(should_end: bool) -> Response {
        Response {
            version: Version::V1_0.to_string(),
            session_attributes: None,
            body: ResBody {
                output_speech: None,
                card: None,
                reprompt: None,
                should_end_session: should_end,
                directives: vec![],
            },
        }
    }

    /// Constructs a basic plain response with a simple card
    pub fn new_simple(title: &str, text: &str) -> Response {
        Response::simple(title, text)
    }

    /// Constructs a basic plain response with a simple card
    pub fn simple(title: &str, text: &str) -> Response {
        Response::new(true)
            .card(Card::simple(title, text))
            .speech(Speech::plain(text))
    }

    /// Constructs an empty response ending the session
    pub fn end() -> Response {
        Response::new(true)
    }

    /// adds a speach element to the response
    pub fn speech(mut self, speech: Speech) -> Self {
        self.body.output_speech = Some(speech);
        self
    }

    /// adds a card to the response
    pub fn card(mut self, card: Card) -> Self {
        self.body.card = Some(card);
        self
    }

    /// adds a directive to the response
    pub fn with_directive(mut self, directive: Directive) -> Self {
        self.body.directives.push(directive);
        self
    }

    /// adds an attribute key/value pair to the response
    /// attributes can be read on the next request for basic state
    /// persistence
    pub fn add_attribute(&mut self, key: &str, val: &str) {
        if let Some(ref mut h) = self.session_attributes {
            let _ = h.insert(String::from(key), String::from(val));
        } else {
            let mut h = HashMap::new();
            h.insert(String::from(key), String::from(val));
            self.session_attributes = Some(h)
        }
    }
}

/// Response struct implementing the [Alexa JSON spec](https://developer.amazon.com/docs/custom-skills/request-and-response-json-reference.html#response-parameters)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Response {
    version: String,
    #[serde(rename = "sessionAttributes")]
    #[serde(skip_serializing_if = "Option::is_none")]
    session_attributes: Option<HashMap<String, String>>,
    #[serde(rename = "response")]
    body: ResBody,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResBody {
    #[serde(rename = "outputSpeech")]
    #[serde(skip_serializing_if = "Option::is_none")]
    output_speech: Option<Speech>,
    #[serde(skip_serializing_if = "Option::is_none")]
    card: Option<Card>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reprompt: Option<Reprompt>,
    #[serde(rename = "shouldEndSession")]
    should_end_session: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    directives: Vec<Directive>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Directive {
    AudioPlayer(AudioPlayerDirective),
}

impl From<AudioPlayerDirective> for Directive {
    fn from(value: AudioPlayerDirective) -> Self {
        Self::AudioPlayer(value)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum DirectiveType {
    AudioPlayerPlay,
    AudioPlayerStop,
    AudioPlayerClearQueue,
    Other(String),
}

impl DirectiveType {
    fn from_str(s: &str) -> Self {
        match s {
            "AudioPlayer.Play" => DirectiveType::AudioPlayerPlay,
            "AudioPlayer.Stop" => DirectiveType::AudioPlayerStop,
            "AudioPlayer.ClearQueue" => DirectiveType::AudioPlayerClearQueue,
            _ => DirectiveType::Other(s.to_string()),
        }
    }
}

impl FromStr for DirectiveType {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(DirectiveType::from_str(s))
    }
}

impl<'a> From<&'a str> for DirectiveType {
    fn from(s: &'a str) -> DirectiveType {
        DirectiveType::from_str(s)
    }
}

impl fmt::Display for DirectiveType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match &*self {
            DirectiveType::AudioPlayerPlay => "AudioPlayer.Play",
            DirectiveType::AudioPlayerStop => "AudioPlayer.Stop",
            DirectiveType::AudioPlayerClearQueue => "AudioPlayer.ClearQueue",
            DirectiveType::Other(str) => str.as_ref(),
        };
        write!(f, "{}", s)
    }
}

impl<'de> Deserialize<'de> for DirectiveType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AudioPlayerDirective {
    #[serde(rename = "type")]
    directive_type: DirectiveType,
    play_behavior: Option<PlayBehavior>,
    audio_item: Option<AudioItem>,
}

impl AudioPlayerDirective {
    pub fn new(directive_type: DirectiveType, play_behavior: PlayBehavior, audio_item: AudioItem) -> Self {
        AudioPlayerDirective {
            directive_type,
            play_behavior: Some(play_behavior),
            audio_item: Some(audio_item),
        }
    }

    pub fn with_directive_type(mut self, dt: DirectiveType) -> Self {
        self.directive_type = dt;
        self
    }

    pub fn with_play_behavior(mut self, pb: PlayBehavior) -> Self {
        self.play_behavior = Some(pb);
        self
    }

    pub fn with_item(mut self, item: AudioItem) -> Self {
        self.audio_item = Some(item);
        self
    }

    /// Adds to the end of the queue
    pub fn enqueue_item(audio_item: AudioItem) -> Self {
        AudioPlayerDirective {
            directive_type: DirectiveType::AudioPlayerPlay,
            play_behavior: Some(PlayBehavior::Enqueue),
            audio_item: Some(audio_item),
        }
    }

    /// Replaces current one and the rest immediately
    pub fn replace_with(audio_item: AudioItem) -> Self {
        AudioPlayerDirective {
            directive_type: DirectiveType::AudioPlayerPlay,
            play_behavior: Some(PlayBehavior::ReplaceAll),
            audio_item: Some(audio_item),
        }
    }

    /// Replaces the next one and the rest of the queue with the given one.
    /// This does not impact the currently playing stream.
    pub fn play_next(audio_item: AudioItem) -> Self {
        AudioPlayerDirective {
            directive_type: DirectiveType::AudioPlayerPlay,
            play_behavior: Some(PlayBehavior::ReplaceAll),
            audio_item: Some(audio_item),
        }
    }

    pub fn stop_player() -> Self {
        AudioPlayerDirective {
            directive_type: DirectiveType::AudioPlayerStop,
            play_behavior: None,
            audio_item: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AudioItem {
    stream: Stream,
    metadata: Option<Metadata>,
}

impl AudioItem {
    pub fn new(stream: Stream) -> Self {
        AudioItem { stream, metadata: None }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct Stream {
    url: String,
    token: Option<String>,
    expected_previous_token: Option<String>,
    offset_in_milliseconds: u64,
    caption_data: Option<CaptionData>,
}

impl Stream {
    pub fn new<T: Into<String>>(url: T) -> Self {
        Stream {
            url: url.into(),
            ..Default::default()
        }
    }

    pub fn with_token<T: Into<String>>(mut self, t: T) -> Self {
        self.token = Some(t.into());
        self
    }

    pub fn with_prev_token<T: Into<String>>(mut self, t: T) -> Self {
        self.expected_previous_token = Some(t.into());
        self
    }

    pub fn with_offset_ms(mut self, offset: u64) -> Self {
        self.offset_in_milliseconds = offset;
        self
    }

    pub fn with_caption(mut self, caption: CaptionData) -> Self {
        self.caption_data = Some(caption);
        self
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    title: Option<String>,
    subtitle: Option<String>,
    art: Option<Artwork>,
    background_image: Option<Artwork>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Artwork {
    sources: Vec<Source>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    url: String,
}


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CaptionData {
    content: Option<String>,
    #[serde(rename = "type")]
    caption_type: Option<String>,
}

// #[serde(default, rename_all = "camelCase")]
// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct CaptionData {
//
// }


enum SpeechType {
    Plain,
    Ssml,
}

impl fmt::Display for SpeechType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            SpeechType::Plain => "PlainText",
            SpeechType::Ssml => "SSML",
        };
        write!(f, "{}", s)
    }
}

/// Play behavior for output speech
#[derive(Debug, Clone, PartialEq)]
pub enum PlayBehavior {
    /// Add the specified stream to the end of the current queue. This does not impact the currently playing stream.
    Enqueue,
    /// Immediately begin playback of the specified stream, and replace current and enqueued streams.
    ReplaceAll,
    /// Replace all streams in the queue. This does not impact the currently playing stream.
    ReplaceEnqueued,
}

impl fmt::Display for PlayBehavior {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            PlayBehavior::Enqueue => "ENQUEUE",
            PlayBehavior::ReplaceAll => "REPLACE_ALL",
            PlayBehavior::ReplaceEnqueued => "REPLACE_ENQUEUED",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for PlayBehavior {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pb = match s {
            "ENQUEUE" => PlayBehavior::Enqueue,
            "REPLACE_ALL" => PlayBehavior::ReplaceAll,
            "REPLACE_ENQUEUED" => PlayBehavior::ReplaceEnqueued,
            _ => PlayBehavior::Enqueue,
        };
        Ok(pb)
    }
}

impl Serialize for PlayBehavior {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for PlayBehavior {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Speech {
    #[serde(rename = "type")]
    speech_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ssml: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "playBehavior")]
    play_behavior: Option<String>,
}

impl Speech {
    /// Constructs a plain text output speech
    pub fn plain(s: &str) -> Speech {
        Speech {
            speech_type: SpeechType::Plain.to_string(),
            text: Some(String::from(s)),
            ssml: None,
            play_behavior: None,
        }
    }

    /// Constructs an SSML output speech (with supplied SSML)
    pub fn ssml(s: &str) -> Speech {
        Speech {
            speech_type: SpeechType::Ssml.to_string(),
            ssml: Some(String::from(s)),
            text: None,
            play_behavior: None,
        }
    }

    /// Adds play behavior to a speech object
    pub fn play_behavior(&mut self, behavior: PlayBehavior) {
        self.play_behavior = Some(behavior.to_string());
    }
}

/// Types of cards for an Alexa response
#[allow(dead_code)]
pub enum CardType {
    Simple,
    Standard,
    LinkAccount,
    AskForPermission,
}

impl fmt::Display for CardType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            CardType::Simple => "Simple",
            CardType::Standard => "Standard",
            CardType::LinkAccount => "LinkAccount",
            CardType::AskForPermission => "AskForPermissonConsent",
        };
        write!(f, "{}", s)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Card {
    #[serde(rename = "type")]
    card_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<Image>,
    #[serde(skip_serializing_if = "Option::is_none")]
    permissions: Option<Vec<String>>,
}

impl Card {
    /// Constructs a simple card for an Alexa repsonse object
    pub fn simple(title: &str, text: &str) -> Card {
        Card {
            card_type: CardType::Simple.to_string(),
            title: Some(String::from(title)),
            content: Some(String::from(text)),
            text: None,
            image: None,
            permissions: None,
        }
    }

    /// Constructs a standard card for an Alexa response object
    pub fn standard(title: &str, text: &str, image: Image) -> Card {
        Card {
            card_type: CardType::Standard.to_string(),
            title: Some(String::from(title)),
            content: None,
            text: Some(String::from(text)),
            image: Some(image),
            permissions: None,
        }
    }

    /// Constructs a link account card for the Alexa response object
    pub fn link_account() -> Card {
        Card {
            card_type: CardType::LinkAccount.to_string(),
            title: None,
            content: None,
            text: None,
            image: None,
            permissions: None,
        }
    }

    /// Constructs a permissions request card with the requested permissions
    pub fn ask_for_permission(permissions: Vec<String>) -> Card {
        Card {
            card_type: CardType::AskForPermission.to_string(),
            title: None,
            content: None,
            text: None,
            image: None,
            permissions: Some(permissions),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Reprompt {
    #[serde(rename = "outputSpeech")]
    output_speech: Speech,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Image {
    #[serde(rename = "smallImageUrl")]
    #[serde(skip_serializing_if = "Option::is_none")]
    small_image_url: Option<String>,
    #[serde(rename = "largeImageUrl")]
    #[serde(skip_serializing_if = "Option::is_none")]
    large_image_url: Option<String>,
}

impl Image {
    pub fn new() -> Image {
        Image::default()
    }

    pub fn small_image_url(mut self, url: String) -> Self {
        self.small_image_url = Some(url);
        self
    }

    pub fn large_image_url(mut self, url: String) -> Self {
        self.large_image_url = Some(url);
        self
    }
}

impl Default for Image {
    fn default() -> Self {
        Image {
            small_image_url: None,
            large_image_url: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        let r = Response::simple("hello, world", "hello, dude");
        assert_eq!(r.version, "1.0");
    }

    #[test]
    fn test_builder() {
        let mut res = Response::new(false)
            .card(Card::standard(
                "foo",
                "bar",
                Image {
                    small_image_url: Some(String::from("baaz.png")),
                    large_image_url: Some(String::from("baazLarge.png")),
                },
            ))
            .speech(Speech::plain("hello"));
        res.add_attribute("attr", "value");
        let t = res.body.card.as_ref().unwrap().title.as_ref().unwrap();
        assert_eq!(t, "foo");
        let txt = res.body.card.as_ref().unwrap().text.as_ref().unwrap();
        assert_eq!(txt, "bar");
        let attr = res
            .session_attributes
            .as_ref()
            .unwrap()
            .get("attr")
            .unwrap();
        assert_eq!(attr, "value");
    }

    #[test]
    fn test_builder_with_image_builder() {
        let mut res = Response::new(false)
            .card(Card::standard(
                "foo",
                "bar",
                Image::new()
                    .small_image_url(String::from("baaz.png"))
                    .large_image_url(String::from("baazLarge.png")),
            ))
            .speech(Speech::plain("hello"));
        res.add_attribute("attr", "value");
        let t = res.body.card.as_ref().unwrap().title.as_ref().unwrap();
        assert_eq!(t, "foo");
        let txt = res.body.card.as_ref().unwrap().text.as_ref().unwrap();
        assert_eq!(txt, "bar");
        let small_img = res
            .body
            .card
            .as_ref()
            .unwrap()
            .image
            .as_ref()
            .unwrap()
            .small_image_url
            .as_ref()
            .unwrap();
        let large_img = res
            .body
            .card
            .as_ref()
            .unwrap()
            .image
            .as_ref()
            .unwrap()
            .large_image_url
            .as_ref()
            .unwrap();

        assert_eq!(small_img, "baaz.png");
        assert_eq!(large_img, "baazLarge.png");

        let attr = res
            .session_attributes
            .as_ref()
            .unwrap()
            .get("attr")
            .unwrap();
        assert_eq!(attr, "value");
    }

    #[test]
    fn test_title() {
        let t = "hello, world";
        let r = Response::simple(t, "hello, dude");

        assert_eq!(r.body.card.unwrap().title.unwrap(), t);
    }

    #[test]
    fn test_text() {
        let t = "hello, dude";
        let r = Response::simple("hello,world", t);

        assert_eq!(r.body.card.unwrap().content.unwrap(), t);
    }

    #[test]
    fn test_should_end() {
        let r = Response::simple("foo", "bar");
        assert_eq!(r.body.should_end_session, true);
    }

    #[test]
    fn test_ap_directive() {
        let s = r#"
        {
        "type": "AudioPlayer.Play",
        "playBehavior": "ENQUEUE",
        "audioItem": {
          "stream": {
            "token": "this-is-the-audio-token",
            "url": "https://my-audio-hosting-site.com/audio/sample-song.mp3",
            "offsetInMilliseconds": 0
          }
        }
      }"#;
        let directive: Result<Directive, _> = serde_json::from_str(s);
        assert!(directive.is_ok());
        let directive = directive.unwrap();
        assert!(matches!(directive, Directive::AudioPlayer{..}));
    }

    #[test]
    fn test_audio_play_response() {}
}
