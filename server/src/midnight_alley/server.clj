(ns midnight-alley.server
  "WebSocket + HTTP front door. The browser receives a JSON snapshot every tick
  and sends {\"type\":\"drop\",\"x\":500} or {\"type\":\"mode\",\"safe\":false}."
  (:require [cheshire.core :as json]
            [clojure.core.async :as a]
            [midnight-alley.static :as static]
            [org.httpkit.server :as http]))

(defn- on-message [cmd-ch msg]
  (try
    (let [{:keys [type x safe]} (json/parse-string msg true)]
      (case type
        "drop" (when (number? x) (a/put! cmd-ch {:type :drop-food :x (double x)}))
        "mode" (when (boolean? safe) (a/put! cmd-ch {:type :set-mode :safe? safe}))
        "reset" (a/put! cmd-ch {:type :reset})
        nil))
    (catch Exception _ nil)))                               ; ignore malformed messages

(defn start!
  "Start the server on `port`, serving the browser build from `public-dir`.
  Returns a function that stops it."
  [{:keys [cmd-ch snapshots]} port public-dir]
  (let [serve-file (static/file-handler public-dir)
        clients (atom #{})
        latest (atom nil)
        tap-ch (a/chan (a/sliding-buffer 1))
        handler (fn [req]
                  (case (:uri req)
                    "/ws" (http/as-channel
                            req
                            {:on-open    (fn [ch] (swap! clients conj ch))
                             :on-close   (fn [ch _] (swap! clients disj ch))
                             :on-receive (fn [_ msg] (on-message cmd-ch msg))})
                    "/snapshot" {:status 200
                                 :headers {"Content-Type" "application/json"
                                           "Access-Control-Allow-Origin" "*"}
                                 :body (or @latest "{}")}
                    "/health" {:status 200 :body "ok"}
                    (or (serve-file (:uri req))
                        {:status 404 :body "not found"})))]
    (a/tap snapshots tap-ch)
    (a/go-loop []
      (when-some [snap (a/<! tap-ch)]
        (let [msg (json/generate-string snap)]
          (reset! latest msg)
          (doseq [ch @clients] (http/send! ch msg)))
        (recur)))
    (let [stop-server (http/run-server handler {:port port})]
      (fn []
        (stop-server)
        (a/untap snapshots tap-ch)
        (a/close! tap-ch)))))
