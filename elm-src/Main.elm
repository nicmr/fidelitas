port module Main exposing (main)

import Browser
import Html exposing (Html, button, div, text, i, progress, p)
import Html.Attributes as Attr
import Html.Attributes exposing (class)
import Html.Events exposing (onClick, onInput)
import Http
import Url.Builder exposing (absolute, crossOrigin)
-- import Json.Encode
import Json.Decode
import Dict exposing (Dict)


import Messages.In
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

type Msg = Play (Maybe String) | Pause | Resume | Stop | VolumeSlider String |  WebsocketIn String

type PlayerState = Paused | Playing | Stopped

-- Model 

type alias Model =
  { volume: Int
  , playerState: PlayerState
  , log: String
  , tracks: Dict String String
  , selected_track: String
  }


init : () -> (Model, Cmd Msg)
init _ =
  (
    { volume = 50
    , playerState = Stopped
    , log = ""
    , tracks = Dict.empty
    -- better initial value possible?
    -- might want to change field to Maybe String
    , selected_track = "0"
    }
    , Cmd.none
  )


-- Update

update : Msg -> Model -> (Model, Cmd Msg)
update msg model =
  case msg of
    -- handle user actions
    Play maybeTrackID->
      case maybeTrackID of
        Just id ->
            ( {model | selected_track = id }
            , websocketOut
                <| Messages.Out.compactJson
                <| Messages.Out.Play
                <| Maybe.withDefault 0
                <| String.toInt id
            )
        Nothing ->
            ( model
            , websocketOut
                <| Messages.Out.compactJson
                <| Messages.Out.Play
                <| Maybe.withDefault 0
                <| String.toInt model.selected_track
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

    -- handle incoming websocket messages
    WebsocketIn value ->
      case Json.Decode.decodeString Messages.In.kindDecoder value of
        Ok kind ->
          let
              payloadDecoder = Messages.In.payloadDecoder kind
          in
            case kind of
              Messages.In.FsState -> case  Json.Decode.decodeString payloadDecoder value of
                Ok payload ->
                  case payload of
                    Messages.In.FsStatePayload new_tracks->
                      ({model | log = model.log ++ value ++ " payloadDecoded", tracks = new_tracks}, Cmd.none)
                    _ -> ({model | log = model.log ++ value ++ "recognised FsState message but payload didn't match"}, Cmd.none)
                Err _ ->  ({model | log = model.log ++ value ++ " error: payloadDecodeError"}, Cmd.none)

              Messages.In.VolumeChange -> case Json.Decode.decodeString payloadDecoder value of 
                Ok payload ->
                  case payload of 
                    Messages.In.VolumeChangePayload new_volume ->
                      ({model | volume = new_volume}, Cmd.none)
                    _ -> ({model | log = model.log ++ value ++ "recognised VolumeChange message but payload didn't match"}, Cmd.none)
                Err _ ->  ({model | log = model.log ++ value ++ " error: payloadDecodeError"}, Cmd.none)

              Messages.In.RegisterSuccess -> ({model | log = model.log ++ value ++ " registerSuccess"}, Cmd.none)
              Messages.In.Play -> ({model | playerState = Playing}, Cmd.none)
              Messages.In.Resume -> ({model | playerState = Playing}, Cmd.none)
              Messages.In.Pause -> ({model | playerState = Paused}, Cmd.none)
              Messages.In.Stop -> ({model | playerState = Stopped}, Cmd.none)
              -- TODO: resume to last correct state on error message
              Messages.In.Error -> ({model | log = model.log ++ value ++ "server informed me invalid command has been sent"}, Cmd.none)
              _ -> (model, Cmd.none)
        Err e -> ({model | log = model.log ++ value ++ " error: can't decode"}, Cmd.none)



-- Subscriptions

subscriptions : Model -> Sub Msg
subscriptions _ =
  websocketIn WebsocketIn


-- View

view : Model -> Html Msg
view model =
  div []
    [ toDivList model.tracks
    , div [ class "log" ]
      [ text <| "Volume: " ++ String.fromInt model.volume
      , div [] [ text "Log:"]
      , div [] [ text model.log]
      ]
    , div [ class "controls" ]
      [ actionsIcons model
      , progress [ Attr.value "20", Attr.max "100"] []
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
    pauseOrPlay = case model.playerState of
      Playing -> i [ class "far fa-pause-circle", onClick Pause] []
      _ -> i [ class "far fa-play-circle", onClick Resume ] []
  in
    div
      [ class "actions" ]
      [ p [ class "currently-playing" ] [ text <| Maybe.withDefault "" <| Dict.get model.selected_track model.tracks ]
      , i [ class "fas fa-chevron-circle-left" ] []
      , pauseOrPlay
      , i [ class "fas fa-chevron-circle-right" ] []
      ]





toDivList : Dict String String -> Html Msg
toDivList dict =
  div
    [ class "tracks"
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