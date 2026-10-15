(ns midnight-alley.sim-test
  (:require [clojure.test :refer [deftest is]]
            [midnight-alley.rat :as rat]
            [midnight-alley.sim :as sim]
            [midnight-alley.world :as w]))

(deftest clock-goes-night-morning-night
  (let [dawn (sim/next-clock {:t 49.95 :morning? false} 0.1)
        next-night (sim/next-clock {:t 11.95 :morning? true} 0.1)]
    (is (:dawn? dawn))
    (is (:morning? dawn))
    (is (:new-night? next-night))
    (is (not (:morning? next-night)))
    (is (= 0.0 (sim/daylight {:t 0.0 :morning? false})))
    (is (= 1.0 (sim/daylight {:t 6.0 :morning? true})))))

(deftest hungry-rat-walks-to-food-and-eats
  (let [world (w/new-world)
        _ (w/drop-food! world 500.0)
        far (assoc (rat/new-rat 1 100.0) :hunger 60.0 :energy 80.0)
        near (assoc (rat/new-rat 2 495.0) :hunger 60.0 :energy 80.0)]
    (is (= 1.0 (:dir (rat/tick far world 0.1 false))))
    (let [fed (rat/tick near world 0.1 false)]
      (is (< (:hunger fed) 20.0))
      (is (= 0 (w/food-count world)))
      (is (= 1 (:eaten @(:diary world)))))))

(deftest second-rat-loses-the-scrap-and-stays-hungry
  (let [world (w/new-world)
        _ (w/drop-food! world 500.0)
        a (assoc (rat/new-rat 1 500.0) :hunger 60.0 :energy 80.0)
        b (assoc (rat/new-rat 2 500.0) :hunger 60.0 :energy 80.0)
        a' (rat/tick a world 0.1 false)
        b' (rat/tick b world 0.1 false)]
    (is (< (:hunger a') 20.0))
    (is (> (:hunger b') 50.0))))

(deftest tired-rat-sleeps-on-a-free-roof-and-only-one-fits
  (let [world (w/new-world)
        x (w/roof-x world 0)
        a (assoc (rat/new-rat 1 x) :hunger 0.0 :energy 10.0)
        b (assoc (rat/new-rat 2 x) :hunger 0.0 :energy 10.0)
        a' (rat/tick a world 0.1 false)
        b' (rat/tick b world 0.1 false)]
    (is (= :sleep (:state a')))
    (is (= 0 (:roof a')))
    (is (= :wander (:state b')))
    (is (= 1 (w/roof-owner world 0)))))

(deftest starving-rat-dies-and-frees-its-roof
  (let [world (w/new-world)
        _ (w/claim-roof! world 0 1)
        r (assoc (rat/new-rat 1 0.0) :state :sleep :roof 0 :hunger 99.9)
        dead (rat/tick r world 0.1 false)]
    (is (false? (:alive? dead)))
    (is (nil? (w/roof-owner world 0)))))

(deftest restart-gives-a-fresh-colony
  (let [world (w/new-world)]
    (sim/spawn-rats! world 3)
    (w/drop-food! world 100.0)
    (w/claim-roof! world 0 99)
    (swap! (:diary world) assoc :night 7 :born 4 :eaten 9)
    (sim/restart! world)
    (is (= w/initial-rats (count @(:rats world))))
    (is (= 0 (w/food-count world)))
    (is (nil? (w/roof-owner world 0)))
    (is (= {:born 0 :eaten 0 :night 1} @(:diary world)))
    (is (= {:t 0.0 :morning? false} @(:clock world)))))

(deftest world-steps-with-agents-and-reports-deaths
  (let [world (w/new-world)]
    (sim/spawn-rats! world 5)
    (doseq [a @(:rats world)] (send a assoc :hunger 100.0))
    (apply await @(:rats world))
    ;; agents are async, so some deaths may be reaped in the first step, the rest in the second
    (let [first-events (sim/step! world 0.1)
          _ (apply await @(:rats world))
          events (into first-events (sim/step! world 0.1))]
      (is (= 5 (count events)))
      (is (every? #(= :death (:type %)) events))
      (is (empty? @(:rats world))))))
