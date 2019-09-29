port module Main exposing (main)

import Browser
import Html exposing (Html, button, div, text)
import Html.Events exposing (onClick)
import Http
import Url.Builder exposing (absolute, crossOrigin)
-- import Json.Encode
import Json.Decode
import Dict exposing (Dict)


import Messages.In

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

type Msg = Increment | Decrement | Play  | Pause | Resume | Stop | WebsocketIn String


-- Model 

type alias Model =
  { counter: Int
  , player_state: PlayerState
  , log: String
  , tracks: Dict String String
  }


init : () -> (Model, Cmd Msg)
init _ =
  (
    { counter = 0
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
    Increment ->
      ( { model | counter = model.counter + 1}, Cmd.none )
    Decrement ->
      ( { model | counter = model.counter - 1}, Cmd.none )
    Play ->
      -- ( model, Http.get { url = absolute ["api", "play", "something"] [], expect = Http.expectString DataReceived})
      (model, websocketOut "Play;something")
    Pause ->
      (model, websocketOut "Pause")
    Resume ->
      (model, websocketOut "Resume")
    Stop ->
      (model, websocketOut "Stop")
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

            _ -> (model, Cmd.none)
        Err e -> ({model | log = model.log ++ value ++ " error: msgKindError"}, Cmd.none)


-- Subscriptions

subscriptions : Model -> Sub Msg
subscriptions model =
  websocketIn WebsocketIn


-- View

view : Model -> Html Msg
view model =
  div []
    [ button [ onClick Decrement ] [ text "-" ]
    , div [] [ text (String.fromInt model.counter) ]
    , button [ onClick Increment ] [ text "+" ]
    , button [ onClick Play] [text "Play"]
    , button [ onClick Pause] [text "Pause"]
    , button [ onClick Resume] [text "Resume"]
    , button [ onClick Stop] [text "Stop"]
    , toHtmlList model.tracks
    , div [] [ text "Log:"]
    , div [] [ text model.log]
    ]

-- View Helpers

toHtmlList : Dict String String -> Html Msg
toHtmlList dict = 
  Html.ul [] (Dict.toList dict |> List.map toLi)

toLi : (String, String) -> Html Msg
toLi (id, name) =
  Html.li [] [text ("ID: " ++ id ++ " Name: " ++ name)]