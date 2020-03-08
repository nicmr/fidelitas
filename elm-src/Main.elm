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
import Messages.In exposing (CurrentMedia, PlaybackState(..))
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
  , log: String
  , allMedia: Dict String String
  }

mediaID : Maybe CurrentMedia -> Maybe Int
mediaID currentMedia =
  Maybe.map (\a -> a.id) currentMedia
  

init : () -> (Model, Cmd Msg)
init _ =
  (
    { volume = 50
    , playbackState = Stopped
    , log = ""
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
          ({model | playbackState = Playing  { currentMedia | progressMillis = min (currentMedia.progressMillis + 1000) currentMedia.lengthMillis} }, Cmd.none)
         _ ->
          (model, Cmd.none)

    -- handle incoming websocket messages
    WebsocketIn value ->
      case Json.Decode.decodeString Messages.In.messageDecoder value of
        Ok kind ->
          case kind of
            Messages.In.PlayerState playbackState allMedia ->
              ({model | log = model.log ++ value ++ " payloadDecoded"
                , allMedia = allMedia
                , playbackState = playbackState
              }, Cmd.none)
            Messages.In.VolumeChange newVolume ->
              ({model | volume = newVolume}, Cmd.none)

            Messages.In.RegisterSuccess ->
              ({model | log = model.log ++ value ++ " registerSuccess"}, Cmd.none)
            -- Messages.In.Play id lengthMillis ->
            --   ({model | playbackState = Playing {id=id, lengthMillis = lengthMillis, progressMillis=0}}, Cmd.none)
            -- -- Messages.In.Resume ->
            -- --   let
            -- --     newPlaybackState = case model.playbackState of
            -- --       Playing a -> Playing a
            -- --       Paused a -> Playing a
            -- --       Stopped -> Stopped
            -- --   in
            -- --     ({model | playbackState = newPlaybackState}, Cmd.none)
            -- Messages.In.Pause -> ({model | playbackState = Paused}, Cmd.none)
            -- Messages.In.Stop -> ({model | playbackState = Stopped}, Cmd.none)

            Messages.In.PlaybackChange newPlaybackState ->
              ({ model | playbackState = newPlaybackState}, Cmd.none)
            -- resume to last correct state on error message? ask server for resync?
            Messages.In.Error ->
              ({model | log = model.log ++ value ++ "server informed me client has sent invalid message"}, Cmd.none)
            _ -> (model, Cmd.none)
        Err _ -> ({model | log = model.log ++ value ++ " error: can't decode"}, Cmd.none)



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
    [ toDivList model.allMedia
    , div [ class "log" ]
      [ p [ class "log-line" ] [ text <| "Volume: " ++ String.fromInt model.volume]
      , p [ class "log-line" ] [ text <| "Track Length in seconds: " ++ ( String.fromInt <| trackLength model.playbackState ) ]
      , div [] [ text "Log:"]
      , div [] [ text model.log]
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

trackLength : PlaybackState -> Int
trackLength playbackState =
  case playbackState of
    Playing media -> media.lengthMillis
    Paused media -> media.lengthMillis
    Stopped -> 0

progressBar : Model -> Html Msg
progressBar model =
  case model.playbackState of
    Playing media -> 
      div []
        [ progress [ Attr.value (String.fromInt (media.progressMillis // 1000)), Attr.max (String.fromInt (media.lengthMillis // 1000)), class "progress-bar"] []
        , text <| "progress: " ++ String.fromInt media.progressMillis ++ " max length: " ++ String.fromInt media.lengthMillis
        ]
    Paused media ->
      div []
        [ progress [ Attr.value (String.fromInt (media.progressMillis // 1000)), Attr.max (String.fromInt (media.lengthMillis // 1000)), class "progress-bar"] []
        , text <| "progress: " ++ String.fromInt media.progressMillis ++ " max length: " ++ String.fromInt media.lengthMillis
        ]
    Stopped ->  progress [ Attr.value "0", Attr.max "100", class "progress-bar"] []




actionsDivs : Html Msg
actionsDivs = 
  div
    [ class "actions"
    ]
    [ div [ onClick (Play Nothing), class "actionButton"]
        [text "Play"]
    , div [ onClick Pause, class "actionButton"]
        [text "Pause"]
    , div [ onClick Resume, class "actionButton"]
        [text "Resume"]
    , div [ onClick Stop, class "actionButton"]
        [text "Stop"]
    ]

actionsIcons : Model -> Html Msg
actionsIcons model =
  let
    pauseOrPlay = case model.playbackState of
      Playing _ -> i [ class "far fa-pause-circle", onClick Pause] []
      _ -> i [ class "far fa-play-circle", onClick Resume ] []
  in
    div
      [ class "actions" ]
      [ p [ class "currently-playing" ]
        [ case model.playbackState of
            Playing currentMedia ->
              currentMedia.id
              |> (\id -> Dict.get (String.fromInt id) model.allMedia)
              |> Maybe.withDefault "track lookup failed"
              |> text
            Paused currentMedia -> 
              currentMedia.id
              |> (\id -> Dict.get (String.fromInt id) model.allMedia)
              |> Maybe.withDefault "track lookup failed"
              |> text
            Stopped ->
              text "None"
        ]
      , i [ class "fas fa-chevron-circle-left" ] []
      , pauseOrPlay
      , i [ class "fas fa-chevron-circle-right" ] []
      ]

toDivList : Dict String String -> Html Msg
toDivList dict =
  div
    [ class "mediaList"
    ]
    (Dict.toList dict |> List.map toClickableDiv)

toClickableDiv : (String, String) -> Html Msg
toClickableDiv (id, name) =
  div
    [ onClick (Play (Just id))
    , Attr.style "cursor" "pointer" --move to css?
    ]
    [ Html.p []
        [ text <| name
        ]
    ]