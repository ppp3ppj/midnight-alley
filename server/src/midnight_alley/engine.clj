(ns midnight-alley.engine
  "The night cycle: a core.async loop that ticks the world forward, applies
  commands from the player, and publishes a snapshot after every tick."
  (:require [clojure.core.async :as a]
            [midnight-alley.sim :as sim]
            [midnight-alley.world :as w]))

(defn- handle-command! [world {:keys [type x safe?]}]
  (case type
    :drop-food (dotimes [_ 5]
                 (w/drop-food! world (mod (+ x (- (rand 80.0) 40.0)) w/world-width)))
    :set-mode (sim/set-mode! world safe?)
    :reset (sim/restart! world)
    nil))

(defn- drain-commands! [world cmd-ch]
  (loop []
    (when-let [cmd (a/poll! cmd-ch)]
      (handle-command! world cmd)
      (recur))))

(defn start!
  "Start the loop. Returns {:cmd-ch :snapshots :stop!}.
  Put {:type :drop-food :x 500.0} or {:type :set-mode :safe? false} on :cmd-ch.
  Tap :snapshots (a mult) to receive one snapshot per tick."
  ([world] (start! world {}))
  ([world {:keys [tick-ms] :or {tick-ms 100}}]
   (let [cmd-ch (a/chan 64)
         out-ch (a/chan (a/sliding-buffer 1))
         stop-ch (a/chan)]
     (a/go-loop [last-ns (System/nanoTime)]
       (let [[_ port] (a/alts! [stop-ch (a/timeout tick-ms)])]
         (when-not (= port stop-ch)
           (let [now (System/nanoTime)
                 dt (min 0.25 (/ (- now last-ns) 1e9))]
             (drain-commands! world cmd-ch)
             (let [events (sim/step! world dt)]
               (a/put! out-ch (sim/snapshot world events)))
             (recur now)))))
     {:cmd-ch    cmd-ch
      :snapshots (a/mult out-ch)
      :stop!     #(a/close! stop-ch)})))
