(ns midnight-alley.sim
  "Ticking the whole world forward: the night/morning clock, the rat agents,
  births at dawn, and the snapshots sent to the browser."
  (:require [midnight-alley.rat :as rat]
            [midnight-alley.world :as w]))

(defn next-clock
  "Pure clock step. Adds :dawn? / :new-night? on the tick where they happen."
  [{:keys [t morning?]} dt]
  (let [t (+ t dt)]
    (cond
      (and (not morning?) (>= t w/night-len)) {:t 0.0 :morning? true :dawn? true}
      (and morning? (>= t w/morning-len)) {:t 0.0 :morning? false :new-night? true}
      :else {:t t :morning? morning?})))

(defn daylight
  "0 = deep night, 1 = full morning (fades in and out around dawn)."
  [{:keys [t morning?]}]
  (if morning?
    (let [half (/ w/morning-len 2.0)]
      (-> (- 1.0 (/ (Math/abs (- t half)) half)) (max 0.0) (min 1.0)))
    0.0))

(defn spawn-rat! [world x]
  (let [id (w/next-id! world)
        a (agent (rat/new-rat id x)
                 :error-mode :continue
                 :error-handler (fn [_ e] (binding [*out* *err*] (println "rat agent error:" e))))]
    (swap! (:rats world) conj a)
    id))

(defn spawn-rats! [world n]
  (dotimes [_ n] (spawn-rat! world (rand w/world-width))))

(defn- births!
  "At dawn, well-fed rats may have pups."
  [world]
  (doseq [a @(:rats world)
          :let [r @a]
          :when (and (:alive? r)
                     (< (:hunger r) 25.0)
                     (< (rand) 0.2)
                     (< (count @(:rats world)) w/max-rats))]
    (spawn-rat! world (:x r))
    (swap! (:diary world) update :born inc)))

(defn- reap!
  "Remove dead rats; returns a :death event for each (the client plays the boom)."
  [world]
  (let [dead (filterv #(not (:alive? @%)) @(:rats world))]
    (when (seq dead)
      (swap! (:rats world) (fn [rats] (vec (remove (set dead) rats)))))
    (mapv (fn [a] {:type :death :x (:x @a) :roof (:roof @a)}) dead)))

(defn step!
  "Advance the world by `dt` seconds. Rats think concurrently in their agents.
  Returns the events that happened this step."
  [world dt]
  (let [c (next-clock @(:clock world) dt)
        morning? (:morning? c)]
    (reset! (:clock world) (select-keys c [:t :morning?]))
    (when (:new-night? c) (swap! (:diary world) update :night inc))
    (doseq [a @(:rats world)] (send a rat/tick world dt morning?))
    (let [events (reap! world)]
      (when (:dawn? c) (births! world))
      events)))

(defn set-mode!
  "Switch between safe (STM) and broken (no protection) mode."
  [world safe?]
  (reset! (:safe? world) safe?)
  (w/reset-stores! world)
  (doseq [a @(:rats world)]
    (send a #(assoc % :state :wander :roof nil :target-roof nil))))

(defn restart!
  "Start over: night 1, empty food, free roofs, a fresh colony. Keeps the current mode."
  [world]
  ;; retire the old rats first so a queued tick can't claim food or a roof after the wipe
  (let [old @(:rats world)]
    (doseq [a old] (send a assoc :alive? false))
    (apply await-for 1000 old))
  (reset! (:rats world) [])
  (w/reset-stores! world)
  (reset! (:diary world) {:born 0 :eaten 0 :night 1})
  (reset! (:clock world) {:t 0.0 :morning? false})
  (spawn-rats! world w/initial-rats))

(defn snapshot
  "Everything the browser needs to draw one frame."
  [world events]
  (let [clock @(:clock world)
        rats (->> @(:rats world) (map deref) (filter :alive?))]
    {:safe?      (w/safe? world)
     :night      (:night @(:diary world))
     :morning?   (:morning? clock)
     :daylight   (daylight clock)
     :food-count (w/food-count world)
     :food       (mapv (fn [[id x]] {:id id :x x}) (w/food-snapshot world))
     :roofs      (w/roofs-snapshot world)
     :rats       (mapv #(-> (select-keys % [:id :x :dir :hunger :energy :state :roof])
                            (assoc :mood (rat/mood %)))
                       rats)
     :diary      (assoc @(:diary world) :alive (count rats))
     :events     events}))
