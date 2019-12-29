port module Main exposing (main)

import Browser
import Html exposing (Html, button, div, text)
import Html.Attributes as Attr
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


-- Model 

type alias Model =
  { volume: Int
  , player_state: PlayerState
  , log: String
  , tracks: Dict String String
  , selected_track: String
  }


init : () -> (Model, Cmd Msg)
init _ =
  (
    { volume = 50
    , player_state = Stopped
    , log = ""
    , tracks = Dict.empty
    -- better initial value possible?
    -- might want to change field to Maybe String
    , selected_track = "0"
    }
    , Cmd.none
  )

type PlayerState = Paused | Playing | Stopped

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
              Messages.In.Play -> ({model | player_state = Playing}, Cmd.none)
              Messages.In.Resume -> ({model | player_state = Playing}, Cmd.none)
              Messages.In.Pause -> ({model | player_state = Paused}, Cmd.none)
              Messages.In.Stop -> ({model | player_state = Stopped}, Cmd.none)
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
    [ button [ onClick (Play Nothing)] [text "Play"]
    , button [ onClick Pause] [text "Pause"]
    , button [ onClick Resume] [text "Resume"]
    , button [ onClick Stop] [text "Stop"]
    -- , toHtmlList model.tracks
    , toDivList model.tracks
    
    , div []
      [ Html.input
        [ Attr.type_ "range"
        , Attr.min "0"
        , Attr.max "125"
        , Attr.value <| String.fromInt model.volume
        , onInput VolumeSlider
        ] []
      , text <| String.fromInt model.volume
      , div [] [ text "Log:"]
      , div [] [ text model.log]
      ]
    ]

-- View Helpers

toDivList : Dict String String -> Html Msg
toDivList dict =
  div [] (Dict.toList dict |> List.map toClickableDiv)

toClickableDiv : (String, String) -> Html Msg
toClickableDiv (id, name) =
  div
    [ onClick (Play (Just id))
    , Attr.style "cursor" "pointer"
    ]
    [ Html.p []
        [ text <| name
        ]
    ]