port module Main exposing (main)

import Browser
import Html exposing (Html, button, div, text)
import Html.Events exposing (onClick)
import Http
import Url.Builder exposing (absolute, crossOrigin)
import Json.Encode
import Json.Decode exposing (field, string, succeed, fail, Decoder)

main =
  Browser.element
  { init = init
  , update = update
  , view = view 
  , subscriptions = subscriptions
  }


port websocketIn : (String -> msg) -> Sub msg
port websocketOut : String -> Cmd msg
-- Msg

type Msg = Increment | Decrement | Play  | Pause | Resume | Stop | WebsocketIn String


-- Model 

type alias Model =
  { counter: Int
  , player_state: PlayerState
  , responses: List String
  , last: IncomingWsMessageKind
  , log: String
  }

type PlayerState = Paused

init : () -> (Model, Cmd Msg)
init _ =
  (
    { counter = 0
    , player_state = Paused
    , responses = []
    , last = KindDefault
    , log = ""
    }
    , Cmd.none
  )


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
      case Json.Decode.decodeString kindFieldDecoder value of
        Ok kind ->
          case kind of
            KindFsState -> ({model | log = model.log ++ value ++ "\nkindfsstate detected\n"}, Cmd.none)
            KindDefault -> ({model | log = model.log ++ value ++ "\nkinddefault detected\n"}, Cmd.none)
        Err e -> ({model | log = model.log ++ value ++ "\nerror detected\n"}, Cmd.none)

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
    , div [] [ text "Log:"]
    , div [] [ text model.log]
    ]


-- TODO: Split into different file

-- type alias IncomingWsMessage = {
--   info: String
--   , kind:  IncomingWsMessageKind
-- }

type IncomingWsMessageKind = KindFsState | KindDefault

type IncomingWsMessage = FsState String | Default

type alias MediaItem =
  {
    id: Int
    , track: String
  }





kindFieldDecoder: Decoder IncomingWsMessageKind
kindFieldDecoder = field "kind" kindDecoder

kindDecoder: Decoder IncomingWsMessageKind
kindDecoder = Json.Decode.string |> Json.Decode.andThen kindFromString

kindFromString: String -> Decoder IncomingWsMessageKind
kindFromString string =
  case string of
    "FsState" -> succeed KindFsState
    _ -> fail <| "Cannot decode"