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

type Msg = Play  | Pause | Resume | Stop | WebsocketIn String | VolumeSlider String


-- Model 

type alias Model =
  { volume: Int
  , player_state: PlayerState
  , log: String
  , tracks: Dict String String
  }


init : () -> (Model, Cmd Msg)
init _ =
  (
    { volume = 0
    , player_state = Stopped
    , log = ""
    , tracks = Dict.empty
    }
    , Cmd.none
  )

type PlayerState = Paused | Playing | Stopped

-- Update

update : Msg -> Model -> (Model, Cmd Msg)
update msg model =
  case msg of
    -- handle user actions
    Play ->
      (model, websocketOut <| Messages.Out.compactJson <| Messages.Out.Play 0)
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
      -- -- TODO: Uncomment when server support is ready. Make sure 
      , websocketOut <| Messages.Out.compactJson <| Messages.Out.VolumeChange volumeInt)
      -- , Cmd.none)

    -- handle incoming websocket messages
    WebsocketIn value ->
      case Json.Decode.decodeString Messages.In.kindDecoder value of
        Ok kind ->
          case kind of
            Messages.In.FsState -> case  Json.Decode.decodeString (Messages.In.payloadDecoder kind) value of
              Ok payload ->
                case payload of
                  Messages.In.FsStatePayload new_tracks->
                    ({model | log = model.log ++ value ++ " payloadDecoded", tracks = new_tracks}, Cmd.none)
                  _ -> ({model | log = model.log ++ value ++ " this shouldn't happen"}, Cmd.none)
              Err e ->  ({model | log = model.log ++ value ++ " error: payloadDecodeError"}, Cmd.none)
            Messages.In.RegisterSuccess -> ({model | log = model.log ++ value ++ " registerSuccess"}, Cmd.none)
            Messages.In.Play -> ({model | player_state = Playing}, Cmd.none)
            Messages.In.Resume -> ({model | player_state = Playing}, Cmd.none)
            Messages.In.Pause -> ({model | player_state = Paused}, Cmd.none)
            Messages.In.Stop -> ({model | player_state = Stopped}, Cmd.none)
            -- TODO: resume to correct state on error message
            Messages.In.Error -> ({model | log = model.log ++ value ++ "server informed me invalid command has been sent"}, Cmd.none)
            _ -> (model, Cmd.none)
        Err e -> ({model | log = model.log ++ value ++ " error: can't decode"}, Cmd.none)



-- Subscriptions

subscriptions : Model -> Sub Msg
subscriptions model =
  websocketIn WebsocketIn


-- View

view : Model -> Html Msg
view model =
  div []
    [ button [ onClick Play] [text "Play"]
    , button [ onClick Pause] [text "Pause"]
    , button [ onClick Resume] [text "Resume"]
    , button [ onClick Stop] [text "Stop"]
    , toHtmlList model.tracks
    , div [] [ text "Log:"]
    , div [] [ text model.log]
    , div []
      [ Html.input
        [ Attr.type_ "range"
        , Attr.min "0"
        , Attr.max "125"
        , Attr.value <| String.fromInt model.volume
        , onInput VolumeSlider
        ] []
      , text <| String.fromInt model.volume
      ]
    ]

-- View Helpers

toHtmlList : Dict String String -> Html Msg
toHtmlList dict = 
  Html.ul [] (Dict.toList dict |> List.map toLi)

toLi : (String, String) -> Html Msg
toLi (id, name) =
  Html.li [] [text ("ID: " ++ id ++ " Name: " ++ name)]