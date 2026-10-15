(ns midnight-alley.rat
  "A rat's mind. Each rat lives in an agent holding a plain map; `tick` is the
  function sent to the agent every world step. Rats decide on their own:
  hunt food, find a warm roof, sleep, or wander."
  (:require [midnight-alley.world :as w]))

(def speed 75.0)         ; world units per second
(def hungry-at 40.0)
(def starve-at 100.0)
(def tired-below 30.0)

(defn new-rat [id x]
  {:id id :x x
   :dir (if (< (rand) 0.5) -1.0 1.0)
   :hunger (rand 40.0)
   :energy (+ 50.0 (rand 50.0))
   :state :wander                ; :wander | :sleep
   :roof nil                     ; roof it sleeps on
   :target-roof nil              ; roof it is walking to
   :turn-timer 0.0
   :alive? true})

(defn mood [{:keys [hunger energy state]}]
  (cond (= state :sleep) :sleeping
        (> hunger 70.0) :starving
        (> hunger hungry-at) :hungry
        (< energy tired-below) :sleepy
        :else :happy))

(defn- sign [d] (if (neg? d) -1.0 1.0))
(defn- dist [a b] (Math/abs (double (- a b))))

(defn- nearest-food [foods x]
  (when (seq foods)
    (apply min-key (fn [[_ fx]] (dist fx x)) (seq foods))))

(defn- nearest-free-roof [world x]
  (let [free (filter #(nil? (w/roof-owner world %)) (range (w/roof-count world)))]
    (when (seq free)
      (apply min-key #(dist (w/roof-x world %) x) free))))

(defn- wander [rat dt]
  (let [t (- (:turn-timer rat) dt)]
    (if (<= t 0.0)
      (assoc rat :dir (if (< (rand) 0.5) -1.0 1.0) :turn-timer (+ 1.0 (rand 2.0)))
      (assoc rat :turn-timer t))))

(defn- hunt [world rat dt]
  (if-let [[fid fx] (nearest-food (w/food-snapshot world) (:x rat))]
    (if (< (dist fx (:x rat)) 10.0)
      (if (w/claim-food! world fid)
        (do (w/note-eaten! world)
            (update rat :hunger #(max 0.0 (- % 50.0))))
        rat)                                                ; lost the race: stays hungry, looks again next tick
      (assoc rat :dir (sign (- fx (:x rat)))))
    (wander rat dt)))                                       ; no food anywhere

(defn- seek-roof [world rat]
  (let [target (:target-roof rat)
        target (if (and target (some? (w/roof-owner world target))) nil target) ; someone took it
        target (or target (nearest-free-roof world (:x rat)))]
    (if-not target
      (assoc rat :target-roof nil)
      (let [cx (w/roof-x world target)]
        (if (< (dist (:x rat) cx) 8.0)
          (if (w/claim-roof! world target (:id rat))
            (assoc rat :x cx :state :sleep :roof target :target-roof nil)
            (assoc rat :target-roof nil))                   ; beaten to it, pick another next tick
          (assoc rat :target-roof target :dir (sign (- cx (:x rat)))))))))

(defn- sleep-step [world rat dt morning?]
  (let [rat (update rat :energy #(min 100.0 (+ % (* 20.0 dt))))]
    (if (and (>= (:energy rat) 100.0) (not morning?))
      (do (w/release-roof! world (:roof rat) (:id rat))
          (assoc rat :state :wander :roof nil))
      rat)))

(defn- die [world rat]
  (when (= :sleep (:state rat))
    (w/release-roof! world (:roof rat) (:id rat)))
  (assoc rat :alive? false))

(defn- move [rat dt]
  (update rat :x #(mod (+ % (* (:dir rat) speed dt)) w/world-width)))

(defn- awake-step [world rat dt morning?]
  (cond
    (> (:hunger rat) hungry-at) (hunt world rat dt)
    (or (< (:energy rat) tired-below) morning?) (seek-roof world rat)
    :else (wander rat dt)))

(defn tick
  "Agent action: advance one rat by `dt` seconds and return its new state."
  [rat world dt morning?]
  (let [rat (update rat :hunger + (* 4.0 dt))]
    (cond
      (not (:alive? rat)) rat
      (>= (:hunger rat) starve-at) (die world rat)
      (= :sleep (:state rat)) (sleep-step world rat dt morning?)
      :else (let [rat (awake-step world (update rat :energy - (* 3.0 dt)) dt morning?)]
              (if (= :sleep (:state rat)) rat (move rat dt))))))
