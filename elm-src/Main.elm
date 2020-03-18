port module Main exposing (main)

import Browser
import Html exposing (Html, div, text, i, progress, p)
import Html.Attributes as Attr
import Html.Attributes exposing (class)
import Html.Events exposing (onClick, onInput)
import Json.Decode
import Dict exposing (Dict)
import Time

import Messages.In
import Messages.In exposing (CurrentMedia, PlaybackState(..), MediaMeta)
import Messages.Out

main =
  Browser.element
  { init = init
  , update = update
  , view = view 
  , subscriptions = subscriptions
  }

-- Ports

port websocketIn : (String -> msg) -> Sub msg
port websocketOut : String -> Cmd msg


-- Msg

type Msg = Play (Maybe String) | Pause | Resume | Stop | VolumeSlider String |  WebsocketIn String | PlayerTick

-- Model 

type alias Model =
  { volume: Int
  , playbackState: PlaybackState
  , log: List String
  , allMedia: Dict String MediaMeta
  }

mediaID : Maybe CurrentMedia -> Maybe Int
mediaID currentMedia =
  Maybe.map (\a -> a.id) currentMedia
  

init : () -> (Model, Cmd Msg)
init _ =
  (
    { volume = 50
    , playbackState = Stopped
    , log = []
    , allMedia = Dict.empty
    -- better initial value possible?
    -- might want to change field to Maybe String
    }
    , Cmd.none
  )


-- Update

update : Msg -> Model -> (Model, Cmd Msg)
update msg model =
  case msg of
    -- handle user actions
    Play maybeID->
      case Maybe.andThen (\id -> String.toInt id) maybeID of
        Just id ->
          -- ( {model | currentMedia = Just {id=id, lengthMillis=0, progressMillis=0 } } --lengthMillis info needs to be delivered by server
          ( model
          , websocketOut
              <| Messages.Out.compactJson
              <| Messages.Out.Play id
            )
        Nothing ->
          let
            id = case model.playbackState of
              Playing media -> media.id
              Paused media -> media.id
              Stopped -> 0
          in
            ( model
            , websocketOut
                <| Messages.Out.compactJson
                <|  Messages.Out.Play id
            )
    Pause ->
      (model, websocketOut <| Messages.Out.compactJson <| Messages.Out.Pause)
    Resume ->
      (model, websocketOut <| Messages.Out.compactJson <| Messages.Out.Resume)
    Stop ->
      (model, websocketOut <| Messages.Out.compactJson <| Messages.Out.Stop)
    VolumeSlider vol_str ->
      let
        volumeInt = String.toInt vol_str |> Maybe.withDefault model.volume
      in
      ({model | volume = volumeInt}
      , websocketOut <| Messages.Out.compactJson <| Messages.Out.VolumeChange volumeInt)
      -- , Cmd.none)
    PlayerTick ->
      case model.playbackState of
         Playing currentMedia -> 
          ({model | playbackState = Playing  { currentMedia | progress = min (currentMedia.progress + 1) (currentMedia.lengthMillis // 1000 ) } }, Cmd.none)
         _ ->
          (model, Cmd.none)

    -- handle incoming websocket messages
    WebsocketIn value ->
      case Json.Decode.decodeString Messages.In.messageDecoder value of
        Ok kind ->
          case kind of
            Messages.In.PlayerState playbackState allMedia ->
              ({model | log =  ("PlayerState: " ++  value) :: model.log
                , allMedia = allMedia
                , playbackState = playbackState
              }, Cmd.none)

            Messages.In.VolumeChange newVolume ->
              ({model | volume = newVolume}, Cmd.none)

            Messages.In.RegisterSuccess ->
              ({model | log = ("RegisterSuccess " ++ value) :: model.log}, Cmd.none)

            Messages.In.PlaybackChange newPlaybackState ->
              ({ model | playbackState = newPlaybackState,
                         log = "PlaybackState:" :: model.log
              }, Cmd.none)

            -- resume to last correct state on error message? ask server for resync?
            Messages.In.Error ->
              ({model | log = "Error:" :: model.log}, Cmd.none)
            _ -> (model, Cmd.none)
        Err _ -> ({model | log = ("Decoding Error:" ++ value) :: model.log}, Cmd.none)



-- Subscriptions

subscriptions : Model -> Sub Msg
subscriptions model =
  let
    -- will tick every second if the media is being played, to be used for progress bar
    playbackTicker = case model.playbackState of
      Playing _ -> [ Time.every 1000 (\_ -> PlayerTick)] --consider onAnimationframe instead
      _ -> []
  in
    Sub.batch <| [websocketIn WebsocketIn] ++ playbackTicker


-- View

view : Model -> Html Msg
view model =
  div []
    [ mediaDivList model.allMedia
    , div [ class "log" ]
      [ p [ class "log-line" ] [ text <| "Volume: " ++ String.fromInt model.volume]
      , p [ class "log-line" ] [ text <| "Track Length: " ++ (Maybe.withDefault "0" <| Maybe.map asMinutes<| currentMediaLength model) ]
      , p [ class "log-line" ] [ text <| "Count of available media: " ++ String.fromInt (Dict.size  model.allMedia)]
      , p [ class "log-line" ] [ text <| "Current state of player: " ++ viewPlaybackState model.playbackState ]
      , div [] [ text "Message log:"]
      , viewLog model.log
      ]
    , div [ class "controls" ]
      [ actionsIcons model
      , progressBar model
      , div []
        [ Html.input
          [ Attr.type_ "range"
          , Attr.min "0"
          , Attr.max "125"
          , Attr.step "5"
          , Attr.value <| String.fromInt model.volume
          , onInput VolumeSlider
          ] []
        ]
      ]
    ]

-- View Helpers

viewLog : List String -> Html Msg
viewLog loglines =
  div [] (List.foldr (\line acc -> p [ class "log-line" ] [ text <| line] :: acc) [] loglines) 


viewPlaybackState : PlaybackState -> String
viewPlaybackState pbs =
  case pbs of
    Playing _ -> "Playing"
    Paused _ -> "Paused"
    Stopped -> "Stopped"

progressBar : Model -> Html Msg
progressBar model =
  case model.playbackState of
    Playing media ->
      let
        trackLength = Maybe.withDefault 0 ( currentMediaLength model )
        trackLengthMinutes = asMinutes trackLength

      in
        div []
          [ progress
            [ Attr.value (String.fromInt media.progress)
            , Attr.max (String.fromInt trackLength)
            , class "progress-bar"
            ] []
          , text <| "progress: " ++ String.fromInt media.progress ++ " max length: " ++ trackLengthMinutes
          ]
    Paused media ->
      let
        trackLengthStr =
          mediaLength model media.id
          |> Maybe.map (\len -> String.fromInt len)
          |> Maybe.withDefault "0"
      in
        div []
          [ progress
            [ Attr.value (String.fromInt media.progress)
            , Attr.max trackLengthStr
            , class "progress-bar"
            ] []
          , text <| "progress: " ++ String.fromInt media.progress ++ " max length: " ++ trackLengthStr
          ]
    Stopped ->  progress [ Attr.value "0", Attr.max "100", class "progress-bar"] []



{-| Returns the length in SECONDS of the media Item with the specified id
Returns Nothing of no media is found for the specified id
-}
mediaLength : Model -> Int -> Maybe Int
mediaLength model id =
  String.fromInt id
    |> (\id_ -> Dict.get id_ model.allMedia)
    |> Maybe.map (\m -> m.length)
    |> Maybe.map (\lenMillis -> lenMillis // 1000)  -- convert to seconds

currentMediaLength : Model -> Maybe Int
currentMediaLength model =
  case model.playbackState of
    Playing media -> mediaLength model media.id
    Paused media -> mediaLength model media.id
    Stopped -> Nothing

{-| Returns a MM:SS representation of the passed seconds
-}
asMinutes : Int -> String
asMinutes totalSeconds =
  let
    mins = totalSeconds // 60
    secs = remainderBy 60 totalSeconds
  in
    String.fromInt mins ++ ":" ++ String.fromInt secs

actionsIcons : Model -> Html Msg
actionsIcons model =
  let
    pauseOrPlay = case model.playbackState of
      Playing _ -> i [ class "far fa-pause-circle", onClick Pause] []
      Paused _ -> i [class "far fa-play-circle", onClick Resume] []
      Stopped -> i [ class "far fa-play-circle", onClick (Play Nothing) ] []
  in
    div
      [ class "actions" ]
      [ p [ class "currently-playing" ]
        [ case model.playbackState of
            Playing currentMedia ->
              currentMedia.id
              |> (\id -> Dict.get (String.fromInt id) model.allMedia)
              |> Maybe.map viewMediaMeta
              |> Maybe.withDefault (div [class "mediameta"] [])
            Paused currentMedia -> 
              currentMedia.id
              |> (\id -> Dict.get (String.fromInt id) model.allMedia)
              |> Maybe.map viewMediaMeta
              |> Maybe.withDefault (div [class "mediameta"] [])
            Stopped ->
              div [class "mediameta"] [text "<no active media>"]
        ]
      , i [ class "fas fa-chevron-circle-left" ] []
      , pauseOrPlay
      , i [ class "fas fa-chevron-circle-right" ] []
      ]

viewMediaMeta : MediaMeta -> Html Msg
viewMediaMeta media =
  div [ class "mediameta"]
    [ text <| "Title: " ++ media.title
    -- , text <| "Album: " ++ media.album
    -- , text <| "Artist: " ++ media.artist
    ]


mediaDivList : Dict String MediaMeta -> Html Msg
mediaDivList dict =
  div
    [ class "mediaList"
    ]
    (Dict.toList dict |> List.map toClickableDiv)

toClickableDiv : (String, MediaMeta) -> Html Msg
toClickableDiv (id, media) =
  div
    [ onClick (Play (Just id))
    , Attr.style "cursor" "pointer" --move to css?
    ]
    [ Html.p []
        [ text <| media.title
        ]
    ]