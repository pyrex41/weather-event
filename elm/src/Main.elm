module Main exposing (main)

import Api
import Browser
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Json.Decode as Decode
import Time
import Types exposing (..)
import WebSocket


main : Program () Model Msg
main =
    Browser.element
        { init = init
        , update = update
        , subscriptions = subscriptions
        , view = view
        }


init : () -> ( Model, Cmd Msg )
init _ =
    ( { page = Dashboard
      , bookings = []
      , students = []
      , alerts = []
      , loading = False
      , error = Nothing
      , successMessage = Nothing
      , bookingFormErrors = []
      , studentFormErrors = []
      , newBookingForm = emptyBookingForm
      , newStudentForm = emptyStudentForm
      , websocketStatus = Connecting
      }
    , Cmd.batch
        [ Api.getBookings GotBookings
        , Api.getStudents GotStudents
        ]
    )


emptyBookingForm : BookingForm
emptyBookingForm =
    { studentId = ""
    , aircraftType = ""
    , scheduledDate = ""
    , endTime = ""
    , locationName = ""
    , locationLat = ""
    , locationLon = ""
    }


emptyStudentForm : StudentForm
emptyStudentForm =
    { name = ""
    , email = ""
    , phone = ""
    , trainingLevel = ""
    }


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        ChangePage page ->
            ( { model | page = page }, Cmd.none )

        GotBookings result ->
            case result of
                Ok bookings ->
                    ( { model | bookings = bookings, loading = False }, Cmd.none )

                Err err ->
                    ( { model | error = Just err, loading = False }, Cmd.none )

        GotStudents result ->
            case result of
                Ok students ->
                    ( { model | students = students, loading = False }, Cmd.none )

                Err err ->
                    ( { model | error = Just err, loading = False }, Cmd.none )

        CreateBooking ->
            let
                errors = validateBookingForm model.newBookingForm
            in
            if List.isEmpty errors then
                ( { model
                    | loading = True
                    , bookingFormErrors = []
                    , successMessage = Nothing
                  }
                , Api.createBooking model.newBookingForm BookingCreated
                )
            else
                ( { model | bookingFormErrors = errors }, Cmd.none )

        BookingCreated result ->
            case result of
                Ok booking ->
                    ( { model
                        | bookings = booking :: model.bookings
                        , newBookingForm = emptyBookingForm
                        , loading = False
                        , successMessage = Just "Booking created successfully"
                        , bookingFormErrors = []
                      }
                    , Cmd.none
                    )

                Err err ->
                    ( { model
                        | error = Just err
                        , loading = False
                        , bookingFormErrors = [ { field = "general", message = err } ]
                      }
                    , Cmd.none
                    )

        CreateStudent ->
            let
                errors = validateStudentForm model.newStudentForm
            in
            if List.isEmpty errors then
                ( { model
                    | loading = True
                    , studentFormErrors = []
                    , successMessage = Nothing
                  }
                , Api.createStudent model.newStudentForm StudentCreated
                )
            else
                ( { model | studentFormErrors = errors }, Cmd.none )

        StudentCreated result ->
            case result of
                Ok student ->
                    ( { model
                        | students = student :: model.students
                        , newStudentForm = emptyStudentForm
                        , loading = False
                        , successMessage = Just "Student created successfully"
                        , studentFormErrors = []
                      }
                    , Cmd.none
                    )

                Err err ->
                    ( { model
                        | error = Just err
                        , loading = False
                        , studentFormErrors = [ { field = "general", message = err } ]
                      }
                    , Cmd.none
                    )

        UpdateBookingForm field value ->
            let
                form =
                    model.newBookingForm

                newForm =
                    case field of
                        StudentIdField ->
                            { form | studentId = value }

                        AircraftTypeField ->
                            { form | aircraftType = value }

                        ScheduledDateField ->
                            { form | scheduledDate = value }

                        EndTimeField ->
                            { form | endTime = value }

                        LocationNameField ->
                            { form | locationName = value }

                        LocationLatField ->
                            { form | locationLat = value }

                        LocationLonField ->
                            { form | locationLon = value }
            in
            ( { model | newBookingForm = newForm }, Cmd.none )

        UpdateStudentForm field value ->
            let
                form =
                    model.newStudentForm

                newForm =
                    case field of
                        NameField ->
                            { form | name = value }

                        EmailField ->
                            { form | email = value }

                        PhoneField ->
                            { form | phone = value }

                        TrainingLevelField ->
                            { form | trainingLevel = value }
            in
            ( { model | newStudentForm = newForm }, Cmd.none )

        WebSocketMessageReceived message ->
            case Decode.decodeString Api.alertDecoder message of
                Ok alert ->
                    ( { model | alerts = alert :: model.alerts }, Cmd.none )

                Err _ ->
                    ( model, Cmd.none )

        WebSocketConnected ->
            ( { model | websocketStatus = Connected }, Cmd.none )

        WebSocketDisconnected ->
            ( { model | websocketStatus = Disconnected }, Cmd.none )

        DismissAlert bookingId ->
            ( { model | alerts = List.filter (\a -> a.bookingId /= bookingId) model.alerts }
            , Cmd.none
            )

        ClearSuccessMessage ->
            ( { model | successMessage = Nothing }, Cmd.none )

        Tick _ ->
            ( model, Cmd.none )


subscriptions : Model -> Sub Msg
subscriptions _ =
    Sub.batch
        [ WebSocket.websocketIn WebSocketMessageReceived
        , WebSocket.websocketConnected (\_ -> WebSocketConnected)
        , WebSocket.websocketDisconnected (\_ -> WebSocketDisconnected)
        , Time.every 10000 Tick
        ]


view : Model -> Html Msg
view model =
    div [ class "app" ]
        [ viewHeader model
        , viewNavigation model
        , viewAlerts model
        , viewContent model
        ]


viewHeader : Model -> Html Msg
viewHeader model =
    header [ class "header" ]
        [ h1 [] [ text "✈️ Flight Schedule Pro" ]
        , div [ class "status" ]
            [ viewWebSocketStatus model.websocketStatus ]
        ]


viewWebSocketStatus : WebSocketStatus -> Html Msg
viewWebSocketStatus status =
    case status of
        Connecting ->
            span [ class "status-badge connecting", attribute "data-testid" "ws-status" ] [ text "Connecting..." ]

        Connected ->
            span [ class "status-badge connected", attribute "data-testid" "ws-status" ] [ text "● Live" ]

        Disconnected ->
            span [ class "status-badge disconnected", attribute "data-testid" "ws-status" ] [ text "○ Disconnected" ]


viewNavigation : Model -> Html Msg
viewNavigation model =
    nav [ class "navigation" ]
        [ button
            [ class (navClass model Dashboard)
            , onClick (ChangePage Dashboard)
            , attribute "data-testid" "nav-dashboard"
            ]
            [ text "Dashboard" ]
        , button
            [ class (navClass model Bookings)
            , onClick (ChangePage Bookings)
            , attribute "data-testid" "nav-bookings"
            ]
            [ text "Bookings" ]
        , button
            [ class (navClass model Students)
            , onClick (ChangePage Students)
            , attribute "data-testid" "nav-students"
            ]
            [ text "Students" ]
        , button
            [ class (navClass model Alerts)
            , onClick (ChangePage Alerts)
            , attribute "data-testid" "nav-alerts"
            ]
            [ text "Alerts" ]
        ]


navClass : Model -> Page -> String
navClass model page =
    if model.page == page then
        "nav-button active"
    else
        "nav-button"


viewAlerts : Model -> Html Msg
viewAlerts model =
    if List.isEmpty model.alerts then
        text ""

    else
        div [ class "alerts-banner" ]
            (List.map viewAlert model.alerts)


viewAlert : Alert -> Html Msg
viewAlert alert =
    div [ class "alert alert-danger" ]
        [ span [] [ text ("⚠️ " ++ alert.message) ]
        , button
            [ class "alert-dismiss"
            , onClick (DismissAlert alert.bookingId)
            ]
            [ text "×" ]
        ]


viewContent : Model -> Html Msg
viewContent model =
    div [ class "content" ]
        [ case model.error of
            Just err ->
                div [ class "error", attribute "data-testid" "error-message" ] [ text ("Error: " ++ err) ]

            Nothing ->
                text ""
        , case model.successMessage of
            Just msg ->
                div [ class "success", attribute "data-testid" "success-message" ]
                    [ text msg
                    , button [ class "success-dismiss", onClick ClearSuccessMessage ] [ text "×" ]
                    ]

            Nothing ->
                text ""
        , case model.page of
            Dashboard ->
                viewDashboard model

            Bookings ->
                viewBookings model

            Students ->
                viewStudents model

            Alerts ->
                viewAlertsPage model
        ]


viewDashboard : Model -> Html Msg
viewDashboard model =
    div [ class "dashboard" ]
        [ h2 [] [ text "Dashboard" ]
        , div [ class "dashboard-stats" ]
            [ div [ class "stat-card", attribute "data-testid" "stat-bookings" ]
                [ h3 [] [ text (String.fromInt (List.length model.bookings)) ]
                , p [] [ text "Total Bookings" ]
                ]
            , div [ class "stat-card", attribute "data-testid" "stat-students" ]
                [ h3 [] [ text (String.fromInt (List.length model.students)) ]
                , p [] [ text "Students" ]
                ]
            , div [ class "stat-card", attribute "data-testid" "stat-alerts" ]
                [ h3 [] [ text (String.fromInt (List.length model.alerts)) ]
                , p [] [ text "Active Alerts" ]
                ]
            ]
        , h3 [] [ text "Recent Bookings" ]
        , viewBookingsList (List.take 5 model.bookings)
        ]


viewBookings : Model -> Html Msg
viewBookings model =
    div [ class "bookings-page" ]
        [ h2 [] [ text "Bookings" ]
        , button
            [ class "button button-primary"
            , onClick (ChangePage Bookings)  -- This will be replaced with a proper show form action
            , attribute "data-testid" "create-booking-btn"
            ]
            [ text "Create New Booking" ]
        , div [ class "form-card" ]
            [ h3 [] [ text "Create New Booking" ]
            , viewBookingForm model
            ]
        , h3 [] [ text "All Bookings" ]
        , viewBookingsList model.bookings
        ]


viewBookingForm : Model -> Html Msg
viewBookingForm model =
    div [ class "form" ]
        [ viewFormErrors model.bookingFormErrors
        , if model.loading then
            div [ class "loading-spinner", attribute "data-testid" "loading-spinner" ] [ text "Loading..." ]
          else
            text ""
        , div [ class "form-group" ]
            [ label [] [ text "Aircraft Type" ]
            , select
                [ onInput (UpdateBookingForm AircraftTypeField)
                , value model.newBookingForm.aircraftType
                , attribute "data-testid" "booking-aircraft"
                ]
                [ option [ value "" ] [ text "Select aircraft type" ]
                , option [ value "Cessna 172" ] [ text "Cessna 172" ]
                , option [ value "Piper Cherokee" ] [ text "Piper Cherokee" ]
                , option [ value "Diamond DA40" ] [ text "Diamond DA40" ]
                ]
            , viewFieldError "aircraft-type" model.bookingFormErrors
            ]
        , div [ class "form-group" ]
            [ label [] [ text "Student" ]
            , select
                [ onInput (UpdateBookingForm StudentIdField)
                , value model.newBookingForm.studentId
                ]
                (option [ value "" ] [ text "Select a student" ]
                    :: List.map
                        (\s -> option [ value s.id ] [ text s.name ])
                        model.students
                )
            , viewFieldError "student" model.bookingFormErrors
            ]
        , div [ class "form-group" ]
            [ label [] [ text "Scheduled Date (YYYY-MM-DDTHH:MM:SSZ)" ]
            , input
                [ type_ "text"
                , placeholder "2024-01-15T14:00:00Z"
                , value model.newBookingForm.scheduledDate
                , onInput (UpdateBookingForm ScheduledDateField)
                , attribute "data-testid" "booking-start-time"
                ]
                []
            , viewFieldError "start-time" model.bookingFormErrors
            ]
        , div [ class "form-group" ]
            [ label [] [ text "End Time (YYYY-MM-DDTHH:MM:SSZ)" ]
            , input
                [ type_ "text"
                , placeholder "2024-01-15T16:00:00Z"
                , value model.newBookingForm.endTime
                , onInput (UpdateBookingForm EndTimeField)
                , attribute "data-testid" "booking-end-time"
                ]
                []
            , viewFieldError "end-time" model.bookingFormErrors
            ]
        , div [ class "form-group" ]
            [ label [] [ text "Location Name" ]
            , input
                [ type_ "text"
                , placeholder "KTOA"
                , value model.newBookingForm.locationName
                , onInput (UpdateBookingForm LocationNameField)
                , attribute "data-testid" "booking-location"
                ]
                []
            , viewFieldError "location" model.bookingFormErrors
            ]
        , div [ class "form-row" ]
            [ div [ class "form-group" ]
                [ label [] [ text "Latitude" ]
                , input
                    [ type_ "text"
                    , placeholder "33.8113"
                    , value model.newBookingForm.locationLat
                    , onInput (UpdateBookingForm LocationLatField)
                    ]
                    []
                ]
            , div [ class "form-group" ]
                [ label [] [ text "Longitude" ]
                , input
                    [ type_ "text"
                    , placeholder "-118.1515"
                    , value model.newBookingForm.locationLon
                    , onInput (UpdateBookingForm LocationLonField)
                    ]
                    []
                ]
            ]
        , button
            [ class "button button-primary"
            , onClick CreateBooking
            , disabled model.loading
            , attribute "data-testid" "submit-booking-btn"
            ]
            [ text
                (if model.loading then
                    "Creating..."

                 else
                    "Create Booking"
                )
            ]
        ]


viewBookingsList : List Booking -> Html Msg
viewBookingsList bookings =
    if List.isEmpty bookings then
        p [ attribute "data-testid" "empty-bookings" ] [ text "No bookings found." ]

    else
        div [ class "bookings-list", attribute "data-testid" "booking-list" ]
            (List.map viewBookingCard bookings)


viewBookingCard : Booking -> Html Msg
viewBookingCard booking =
    div [ class ("booking-card status-" ++ String.toLower booking.status), attribute "data-testid" "booking-item" ]
        [ div [ class "booking-header" ]
            [ h4 [] [ text booking.departureLocation.name ]
            , span [ class "status-badge" ] [ text booking.status ]
            ]
        , p [] [ text ("Aircraft: " ++ booking.aircraftType) ]
        , p [] [ text ("Date: " ++ formatDateWithTimezone booking.scheduledDate) ]
        , p [] [ text ("Student ID: " ++ booking.studentId) ]
        , button
            [ class "button button-secondary"
            , attribute "data-testid" "reschedule-btn"
            , onClick (ChangePage Bookings)  -- Placeholder - reschedule feature not yet implemented
            ]
            [ text "Reschedule" ]
        ]


formatDateWithTimezone : String -> String
formatDateWithTimezone dateStr =
    dateStr ++ " UTC"


viewStudents : Model -> Html Msg
viewStudents model =
    div [ class "students-page" ]
        [ h2 [] [ text "Students" ]
        , button
            [ class "button button-primary"
            , onClick (ChangePage Students)  -- This will be replaced with a proper show form action
            , attribute "data-testid" "create-student-btn"
            ]
            [ text "Add New Student" ]
        , div [ class "form-card" ]
            [ h3 [] [ text "Add New Student" ]
            , viewStudentForm model
            ]
        , h3 [] [ text "All Students" ]
        , viewStudentsList model.students
        ]


viewStudentForm : Model -> Html Msg
viewStudentForm model =
    div [ class "form" ]
        [ viewFormErrors model.studentFormErrors
        , if model.loading then
            div [ class "loading-spinner", attribute "data-testid" "loading-spinner" ] [ text "Loading..." ]
          else
            text ""
         , div [ class "form-group" ]
             [ label [] [ text "Name" ]
             , input
                 [ type_ "text"
                 , placeholder "John Doe"
                 , value model.newStudentForm.name
                 , onInput (UpdateStudentForm NameField)
                 , attribute "data-testid" "student-name"
                 ]
                 []
             , viewFieldError "name" model.studentFormErrors
             ]
         , div [ class "form-group" ]
             [ label [] [ text "Email" ]
             , input
                 [ type_ "email"
                 , placeholder "john@example.com"
                 , value model.newStudentForm.email
                 , onInput (UpdateStudentForm EmailField)
                 , attribute "data-testid" "student-email"
                 ]
                 []
             , viewFieldError "email" model.studentFormErrors
             ]
         , div [ class "form-group" ]
             [ label [] [ text "Phone" ]
             , input
                 [ type_ "tel"
                 , placeholder "(555) 123-4567"
                 , value model.newStudentForm.phone
                 , onInput (UpdateStudentForm PhoneField)
                 , attribute "data-testid" "student-phone"
                 ]
                 []
             , viewFieldError "phone" model.studentFormErrors
             ]
        , div [ class "form-group" ]
            [ label [] [ text "Training Level" ]
            , select
                [ onInput (UpdateStudentForm TrainingLevelField)
                , value model.newStudentForm.trainingLevel
                , attribute "data-testid" "student-training-level"
                ]
                [ option [ value "" ] [ text "Select training level" ]
                , option [ value "STUDENT_PILOT" ] [ text "Student Pilot" ]
                , option [ value "PRIVATE_PILOT" ] [ text "Private Pilot" ]
                , option [ value "INSTRUMENT_RATED" ] [ text "Instrument Rated" ]
                ]
            , viewFieldError "training-level" model.studentFormErrors
            ]
        , button
            [ class "button button-primary"
            , onClick CreateStudent
            , disabled model.loading
            , attribute "data-testid" "submit-student-btn"
            ]
            [ text
                (if model.loading then
                    "Creating..."

                 else
                    "Add Student"
                )
            ]
        ]


viewStudentsList : List Student -> Html Msg
viewStudentsList students =
    if List.isEmpty students then
        p [ attribute "data-testid" "empty-students" ] [ text "No students found." ]

    else
        div [ class "students-list", attribute "data-testid" "student-list" ]
            (List.map viewStudentCard students)


viewStudentCard : Student -> Html Msg
viewStudentCard student =
    div [ class "student-card", attribute "data-testid" "student-item" ]
        [ h4 [] [ text student.name ]
        , p [] [ text ("Email: " ++ student.email) ]
        , p [] [ text ("Phone: " ++ student.phone) ]
        , span [ class "badge" ] [ text student.trainingLevel ]
        ]


viewAlertsPage : Model -> Html Msg
viewAlertsPage model =
    div [ class "alerts-page" ]
        [ h2 [] [ text "Weather Alerts" ]
        , if List.isEmpty model.alerts then
            p [] [ text "No active alerts." ]

          else
            div [ class "alerts-list" ]
                (List.map viewAlertCard model.alerts)
        ]


viewAlertCard : Alert -> Html Msg
viewAlertCard alert =
    div [ class "alert-card" ]
        [ h4 [] [ text ("⚠️ " ++ alert.alertType) ]
        , p [] [ text alert.message ]
        , case alert.studentName of
            Just name ->
                p [] [ text ("Student: " ++ name) ]

            Nothing ->
                text ""
        , case alert.originalDate of
            Just date ->
                p [] [ text ("Original Date: " ++ formatDateWithTimezone date) ]

            Nothing ->
                text ""
        , button
            [ class "button button-secondary"
            , onClick (DismissAlert alert.bookingId)
            ]
            [ text "Dismiss" ]
        ]


viewFormErrors : List FormError -> Html Msg
viewFormErrors errors =
    case List.filter (\e -> e.field == "general") errors of
        [] ->
            text ""

        generalErrors ->
            div [ class "form-errors" ]
                (List.map (\e -> div [ class "error-message" ] [ text e.message ]) generalErrors)


viewFieldError : String -> List FormError -> Html Msg
viewFieldError fieldName errors =
    case List.filter (\e -> e.field == fieldName) errors of
        [] ->
            text ""

        fieldErrors ->
            div [ class "field-error", attribute "data-testid" ("error-" ++ fieldName) ]
                [ text (String.join ", " (List.map .message fieldErrors)) ]


validateStudentForm : StudentForm -> List FormError
validateStudentForm form =
    let
        nameErrors =
            if String.isEmpty (String.trim form.name) then
                [ { field = "name", message = "Name is required" } ]
            else if String.length (String.trim form.name) < 2 then
                [ { field = "name", message = "Name must be at least 2 characters" } ]
            else
                []

        emailErrors =
            if String.isEmpty (String.trim form.email) then
                [ { field = "email", message = "Email is required" } ]
            else if not (String.contains "@" form.email) then
                [ { field = "email", message = "Please enter a valid email address" } ]
            else
                []

        phoneErrors =
            if String.isEmpty (String.trim form.phone) then
                [ { field = "phone", message = "Phone is required" } ]
            else
                []

        trainingLevelErrors =
            if String.isEmpty form.trainingLevel then
                [ { field = "training-level", message = "Training level is required" } ]
            else
                []
    in
    nameErrors ++ emailErrors ++ phoneErrors ++ trainingLevelErrors


validateBookingForm : BookingForm -> List FormError
validateBookingForm form =
    let
        aircraftErrors =
            if String.isEmpty form.aircraftType then
                [ { field = "aircraft-type", message = "Aircraft type is required" } ]
            else
                []

        studentErrors =
            if String.isEmpty form.studentId then
                [ { field = "student", message = "Student selection is required" } ]
            else
                []

        dateErrors =
            if String.isEmpty (String.trim form.scheduledDate) then
                [ { field = "start-time", message = "Start time is required" } ]
            else
                []

        endTimeErrors =
            if String.isEmpty (String.trim form.endTime) then
                [ { field = "end-time", message = "End time is required" } ]
            else if not (String.isEmpty (String.trim form.scheduledDate)) && form.endTime < form.scheduledDate then
                [ { field = "end-time", message = "End time must be after start time" } ]
            else
                []

        locationErrors =
            if String.isEmpty (String.trim form.locationName) then
                [ { field = "location", message = "Location is required" } ]
            else
                []
    in
    aircraftErrors ++ studentErrors ++ dateErrors ++ endTimeErrors ++ locationErrors
