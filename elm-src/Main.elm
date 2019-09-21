port module Main exposing (main)

import Browser
import Html exposing (Html, button, div, text)
import Html.Events exposing (onClick)
import Http
import Url.Builder exposing (absolute, crossOrigin)

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
  , input: String
  }

type PlayerState = Paused

init : () -> (Model, Cmd Msg)
init _ =
  (
    { counter = 0
    , player_state = Paused
    , responses = []
    , input = ""
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
      (model, Cmd.none )


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
    ]
