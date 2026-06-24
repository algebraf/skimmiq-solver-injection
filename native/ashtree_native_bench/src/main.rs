use std::cmp::Ordering;
use std::collections::{BTreeMap, BinaryHeap, HashMap, HashSet, VecDeque};
use std::env;
use std::fmt;
use std::fs::{self, File};
use std::hint::black_box;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

type Color = u8;
type MoveIndex = usize;
type Key = [u16; 9];
type PatternKey = [u8; FACE_COUNT * 6];

const DEFAULT_RESCUE_THRESHOLD: usize = 45;
const DEFAULT_RESCUE_TIME_LIMIT_MS: u64 = 10_000;
const F_RESCUE_LOCAL_WINDOW: usize = 12;
const F_RESCUE_LOCAL_DEPTH: usize = 7;
const DEFAULT_OPERATION_PORTFOLIO_TIME_LIMIT_MS: u64 = 15_000;
const DEFAULT_OPERATION_PORTFOLIO_THRESHOLD: usize = 130;
const DEFAULT_CORRIDOR_PREFIX_LEN: usize = 2;
const DEFAULT_CORRIDOR_QUOTA_PERCENT: usize = 20;
const DEFAULT_PATTERN_DB_DEPTH: usize = 4;
const DEFAULT_PATTERN_DB_WEIGHT: i32 = 300;
const DEFAULT_PATTERN_DB_THRESHOLD: usize = 65;
const DEFAULT_LANDMARK_DEPTH: usize = 24;
const DEFAULT_LANDMARK_WIDTH: usize = 1_200;
const DEFAULT_LANDMARK_CANDIDATES: usize = 8;
const DEFAULT_LANDMARK_TIME_LIMIT_MS: u64 = 60_000;
const DEFAULT_LANDMARK_SUFFIX_TIME_LIMIT_MS: u64 = 5_000;
const DEFAULT_HARD_RESCUE_TIME_LIMIT_MS: u64 = 120_000;
const DEFAULT_HARD_RESCUE_DEPTH: usize = 160;
const DEFAULT_HARD_RESCUE_WIDTH: usize = 2_500;
const DEFAULT_HARD_RESCUE_RESTARTS: usize = 2;
const DEFAULT_PAIR_REGION_TABLE_DEPTH: usize = 6;
const DEFAULT_PAIR_REGION_FORWARD_DEPTH: usize = 6;
const DEFAULT_PAIR_REGION_TIME_LIMIT_MS: u64 = 60_000;
const DEFAULT_PAIR_REGION_SUFFIX_TIME_LIMIT_MS: u64 = 60_000;
const DEFAULT_PAIR_REGION_DEPTH: usize = 120;
const DEFAULT_PAIR_REGION_WIDTH: usize = 1_500;
const DEFAULT_PAIR_REGION_RESTARTS: usize = 3;
const DEFAULT_PAIR_REGION_PREFIXES: usize = 4;
const DEFAULT_PHASE_PREFIXES: usize = 3;
const DEFAULT_PHASE_TIME_LIMIT_MS: u64 = 5_000;
const DEFAULT_PHASE_SUFFIX_TIME_LIMIT_MS: u64 = 10_000;
const DEFAULT_PHASE_PREFIX_SUFFIX_PROBE_TIME_LIMIT_MS: u64 = 5_000;
const DEFAULT_PHASE_PREFIX_SUFFIX_PROBE_CANDIDATES: usize = 4;
const DEFAULT_PHASE_DEPTH: usize = 60;
const DEFAULT_PHASE_WIDTH: usize = 500;
const DEFAULT_PHASE_RESTARTS: usize = 2;
const DEFAULT_PHASE_NEAR_MISSES: usize = 2;
const E_CLASSIC_CORNER_PRIMARY_PREFIXES: usize = 4;
const E_CLASSIC_CORNER_FALLBACK_PREFIXES: usize = 4;
const E_CLASSIC_CORNER_RESCUE_MIN_SCRAMBLE: usize = 20;
const E_CLASSIC_CORNER_QUALITY_THRESHOLD: usize = 80;
const E_CLASSIC_STRONG_LOCAL_THRESHOLD: usize = 40;
const E_CLASSIC_STRONG_LOCAL_WINDOW: usize = 12;
const E_CLASSIC_STRONG_LOCAL_DEPTH: usize = 8;
const E_CLASSIC_NO_RESULT_HARD_RESCUE_DEPTH: usize = 200;
const E_CLASSIC_NO_RESULT_HARD_RESCUE_WIDTH: usize = 4_500;
const E_CLASSIC_NO_RESULT_HARD_RESCUE_RESTARTS: usize = 3;
const E_CLASSIC_QUALITY_HARD_RESCUE_THRESHOLD: usize = 65;
const E_CLASSIC_QUALITY_HARD_RESCUE_TIME_LIMIT_MS: u64 = 45_000;
const DEFAULT_AXIS_RING_RESCUE_THRESHOLD: usize = 60;
const DEFAULT_AXIS_RING_RESCUE_TIME_LIMIT_MS: u64 = 45_000;
const DEFAULT_AXIS_RING_RESCUE_TABLE_DEPTH: usize = 24;
const DEFAULT_AXIS_RING_RESCUE_PATTERN_DEPTH: usize = 1;
const DEFAULT_AXIS_RING_RESCUE_EXPAND_DEPTH: usize = 1;
const DEFAULT_AXIS_RING_RESCUE_DEPTH: usize = 70;
const DEFAULT_AXIS_RING_RESCUE_WIDTH: usize = 3_000;
const DEFAULT_AXIS_RING_RESCUE_RESTARTS: usize = 3;
const DEFAULT_AXIS_RING_RESCUE_CORNER_SKIP_THRESHOLD: usize = 65;
const DEFAULT_PDB_SEED_COUNT: usize = 200;
const DEFAULT_PDB_SEED_STEP_START: usize = 8;
const DEFAULT_PDB_SEED_STEP_END: usize = 14;
const DEFAULT_PDB_RANDOM_WALK_MIN: usize = 8;
const DEFAULT_PDB_RANDOM_WALK_MAX: usize = 12;
const DEFAULT_DIRECTION_SURVIVAL_DEPTH: usize = 8;
const DEFAULT_COMMUTATOR_MAX_LEN: usize = 2;
const DEFAULT_COMMUTATOR_TOP: usize = 40;
const DEFAULT_COMMUTATOR_GREEDY_STEPS: usize = 80;
const DEFAULT_COMMUTATOR_PLATEAU_LOOKAHEAD: usize = 0;
const DEFAULT_COMMUTATOR_SUFFIX_TIME_LIMIT_MS: u64 = 10_000;
const DEFAULT_COMMUTATOR_ENDGAME_DEPTH: usize = 6;
const DEFAULT_COMMUTATOR_ENDGAME_WIDTH: usize = 2_000;
const DEFAULT_COMMUTATOR_ENDGAME_TIME_LIMIT_MS: u64 = 10_000;
const DEFAULT_FEATURE_COST_REPEATS: usize = 1_000;

const WHITE: Color = 0;
const RED: Color = 1;
const BLUE: Color = 2;
const MAGENTA: Color = 3;
const GREEN: Color = 4;
const YELLOW: Color = 5;

const FACE_COUNT: usize = 6;
const TARGET_COLORS: [Color; 6] = [WHITE, RED, BLUE, MAGENTA, GREEN, YELLOW];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum LayoutId {
    A,
    B,
    C,
    D,
    E,
    F,
}

impl LayoutId {
    fn all() -> Vec<Self> {
        vec![Self::A, Self::B, Self::C, Self::D, Self::E, Self::F]
    }

    fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_uppercase().as_str() {
            "A" => Some(Self::A),
            "B" => Some(Self::B),
            "C" => Some(Self::C),
            "D" => Some(Self::D),
            "E" => Some(Self::E),
            "F" => Some(Self::F),
            _ => None,
        }
    }

    fn dims(self) -> Dims {
        match self {
            Self::A => Dims {
                rows: 1,
                cols: 1,
                layers: 2,
            },
            Self::B => Dims {
                rows: 1,
                cols: 1,
                layers: 3,
            },
            Self::C => Dims {
                rows: 2,
                cols: 2,
                layers: 1,
            },
            Self::D => Dims {
                rows: 2,
                cols: 2,
                layers: 2,
            },
            Self::E => Dims {
                rows: 3,
                cols: 3,
                layers: 3,
            },
            Self::F => Dims {
                rows: 3,
                cols: 3,
                layers: 1,
            },
        }
    }

    fn default_table_depth(self) -> usize {
        match self {
            Self::A | Self::B => 8,
            Self::C => 7,
            Self::D => 6,
            Self::E | Self::F => 5,
        }
    }

    fn default_forward_depth(self) -> usize {
        self.default_table_depth()
    }
}

impl fmt::Display for LayoutId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            LayoutId::A => "A",
            LayoutId::B => "B",
            LayoutId::C => "C",
            LayoutId::D => "D",
            LayoutId::E => "E",
            LayoutId::F => "F",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Difficulty {
    Easy,
    Moderate,
    Classic,
}

impl Difficulty {
    fn all() -> Vec<Self> {
        vec![Self::Easy, Self::Moderate, Self::Classic]
    }

    fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "easy" | "e" => Some(Self::Easy),
            "moderate" | "m" => Some(Self::Moderate),
            "classic" | "c" => Some(Self::Classic),
            _ => None,
        }
    }

    fn face_color(self, face: Face) -> Color {
        match self {
            Self::Easy => match face {
                Face::Top | Face::Bottom => RED,
                Face::Front | Face::Back | Face::Left | Face::Right => WHITE,
            },
            Self::Moderate => match face {
                Face::Top | Face::Bottom => WHITE,
                Face::Left | Face::Right => RED,
                Face::Front | Face::Back => GREEN,
            },
            Self::Classic => match face {
                Face::Top => WHITE,
                Face::Right => RED,
                Face::Front => BLUE,
                Face::Left => MAGENTA,
                Face::Back => GREEN,
                Face::Bottom => YELLOW,
            },
        }
    }

    fn required_pairs(self) -> Vec<(Color, Color)> {
        match self {
            Self::Classic => sorted_pairs(&[(BLUE, GREEN), (RED, MAGENTA), (WHITE, YELLOW)]),
            Self::Moderate => sorted_pairs(&[(RED, RED), (WHITE, WHITE), (GREEN, GREEN)]),
            Self::Easy => sorted_pairs(&[(RED, RED), (WHITE, WHITE), (WHITE, WHITE)]),
        }
    }
}

impl fmt::Display for Difficulty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Difficulty::Easy => "easy",
            Difficulty::Moderate => "moderate",
            Difficulty::Classic => "classic",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Clone, Copy)]
struct Dims {
    rows: usize,
    cols: usize,
    layers: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Face {
    Front,
    Back,
    Left,
    Right,
    Top,
    Bottom,
}

const FACES: [Face; FACE_COUNT] = [
    Face::Front,
    Face::Back,
    Face::Left,
    Face::Right,
    Face::Top,
    Face::Bottom,
];

impl Face {
    fn index(self) -> usize {
        match self {
            Face::Front => 0,
            Face::Back => 1,
            Face::Left => 2,
            Face::Right => 3,
            Face::Top => 4,
            Face::Bottom => 5,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Face::Front => "front",
            Face::Back => "back",
            Face::Left => "left",
            Face::Right => "right",
            Face::Top => "top",
            Face::Bottom => "bottom",
        }
    }
}

#[derive(Debug, Clone)]
struct Sticker {
    face: Face,
    x: usize,
    y: usize,
    z: usize,
}

#[derive(Debug, Clone)]
struct Move {
    tape_id: String,
    cycle: Vec<usize>,
    direction: i8,
    axis: u8,
    layer: usize,
    inverse: usize,
}

#[derive(Debug, Clone)]
struct Puzzle {
    layout: LayoutId,
    difficulty: Difficulty,
    stickers: Vec<Sticker>,
    face_indexes: [Vec<usize>; FACE_COUNT],
    moves: Vec<Move>,
    solved_colors: Vec<Color>,
    target_color_counts: [usize; 6],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScrambleProfile {
    Lab,
    AndroidOriginal,
}

impl ScrambleProfile {
    fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "lab" | "solver-lab" => Some(Self::Lab),
            "android-original" | "android" | "original" => Some(Self::AndroidOriginal),
            _ => None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Lab => "lab",
            Self::AndroidOriginal => "android-original",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PdbSeedSource {
    AxisRing,
    AriadneMidpoints,
    RandomWalk,
}

impl PdbSeedSource {
    fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "axis-ring" | "axis_ring" | "axis" | "ring" => Some(Self::AxisRing),
            "ariadne" | "ariadne-midpoints" | "ariadne_midpoints" | "midpoints" | "trace"
            | "traces" => Some(Self::AriadneMidpoints),
            "random-walk" | "random_walk" | "random" | "walk" => Some(Self::RandomWalk),
            _ => None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::AxisRing => "axis-ring",
            Self::AriadneMidpoints => "ariadne-midpoints",
            Self::RandomWalk => "random-walk",
        }
    }

    fn has_known_suffix_hint(self) -> bool {
        matches!(self, Self::AxisRing)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BeamRankMode {
    Score,
    PatternDistance,
    PatternHybrid,
    RingRotation,
    RingHybrid,
    RingPortfolio,
}

impl BeamRankMode {
    fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "score" | "default" => Some(Self::Score),
            "pattern-distance" | "pattern_distance" | "pattern" | "distance" => {
                Some(Self::PatternDistance)
            }
            "pattern-hybrid" | "pattern_hybrid" | "hybrid" | "distance-hybrid"
            | "distance_hybrid" => Some(Self::PatternHybrid),
            "ring-rotation" | "ring_rotation" | "ring" | "rotation" => Some(Self::RingRotation),
            "ring-hybrid" | "ring_hybrid" | "rotation-hybrid" | "rotation_hybrid" => {
                Some(Self::RingHybrid)
            }
            "ring-portfolio" | "ring_portfolio" | "rotation-portfolio" | "rotation_portfolio" => {
                Some(Self::RingPortfolio)
            }
            _ => None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Score => "score",
            Self::PatternDistance => "pattern-distance",
            Self::PatternHybrid => "pattern-hybrid",
            Self::RingRotation => "ring-rotation",
            Self::RingHybrid => "ring-hybrid",
            Self::RingPortfolio => "ring-portfolio",
        }
    }

    fn needs_pattern_db(self) -> bool {
        matches!(self, Self::PatternDistance | Self::PatternHybrid)
    }
}

impl Puzzle {
    fn new(layout: LayoutId, difficulty: Difficulty) -> Self {
        let stickers = build_stickers(layout);
        let sticker_index_by_id = build_sticker_index(&stickers);
        let face_indexes = build_face_indexes(&stickers);
        let tapes = build_tapes(layout, &sticker_index_by_id);
        let moves = build_moves(tapes);
        let solved_colors = stickers
            .iter()
            .map(|sticker| difficulty.face_color(sticker.face))
            .collect::<Vec<_>>();
        let target_color_counts = count_colors(&solved_colors);

        Self {
            layout,
            difficulty,
            stickers,
            face_indexes,
            moves,
            solved_colors,
            target_color_counts,
        }
    }

    fn apply_move(&self, colors: &mut [Color], move_index: MoveIndex) {
        let mv = &self.moves[move_index];
        apply_cycle(colors, &mv.cycle, mv.direction);
    }

    fn apply_moves(&self, colors: &mut [Color], moves: &[MoveIndex]) {
        for &move_index in moves {
            self.apply_move(colors, move_index);
        }
    }

    fn inverse_index(&self, move_index: MoveIndex) -> MoveIndex {
        self.moves[move_index].inverse
    }

    fn find_move(&self, axis: u8, layer: usize, direction: i8) -> Option<MoveIndex> {
        self.moves
            .iter()
            .position(|mv| mv.axis == axis && mv.layer == layer && mv.direction == direction)
    }

    fn move_text(&self, move_index: MoveIndex) -> String {
        let mv = &self.moves[move_index];
        format!("{}{}", mv.tape_id, if mv.direction > 0 { "+" } else { "-" })
    }

    fn moves_text(&self, moves: &[MoveIndex]) -> String {
        moves
            .iter()
            .map(|&move_index| self.move_text(move_index))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TargetMode {
    Uniform,
    Android,
    AndroidMultiGoal,
    AndroidPortfolio,
    PairRegion,
}

impl TargetMode {
    fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "uniform" | "upstream" => Some(Self::Uniform),
            "android" | "android-strict" | "strict" => Some(Self::Android),
            "android-multi" | "multi" | "multi-goal" => Some(Self::AndroidMultiGoal),
            "android-portfolio" | "portfolio" => Some(Self::AndroidPortfolio),
            "pair-region" | "pair_region" | "pairs-region" | "pairs_region" => {
                Some(Self::PairRegion)
            }
            _ => None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Uniform => "uniform",
            Self::Android => "android",
            Self::AndroidMultiGoal => "android-multi",
            Self::AndroidPortfolio => "android-portfolio",
            Self::PairRegion => "pair-region",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SolveMethod {
    None,
    Mitm,
    Macro,
}

impl SolveMethod {
    fn label(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Mitm => "mitm",
            Self::Macro => "macro",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OperationProfile {
    Auto,
    Raw,
    Basic,
    Pairs,
    Conjugates,
    Expanded,
    ExpandedParallel,
    ExpandedWide,
}

impl OperationProfile {
    fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "auto" | "default" => Some(Self::Auto),
            "raw" | "moves" | "single" => Some(Self::Raw),
            "basic" | "base" => Some(Self::Basic),
            "pairs" | "pair" => Some(Self::Pairs),
            "conjugates" | "conjugate" | "conj" => Some(Self::Conjugates),
            "expanded" | "all" => Some(Self::Expanded),
            "expanded-parallel" | "expanded_parallel" | "parallel" => Some(Self::ExpandedParallel),
            "expanded-wide" | "expanded_wide" | "wide" => Some(Self::ExpandedWide),
            _ => None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Raw => "raw",
            Self::Basic => "basic",
            Self::Pairs => "pairs",
            Self::Conjugates => "conjugates",
            Self::Expanded => "expanded",
            Self::ExpandedParallel => "expanded-parallel",
            Self::ExpandedWide => "expanded-wide",
        }
    }

    fn for_layout(self, layout: LayoutId) -> Self {
        match (self, layout) {
            (Self::Auto, LayoutId::E) => Self::ExpandedParallel,
            (Self::Auto, _) => Self::Basic,
            (other, _) => other,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AxisRingRescuePosition {
    AfterCascade,
    BeforeCorner,
}

impl AxisRingRescuePosition {
    fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "after-cascade" | "after_cascade" | "after" => Some(Self::AfterCascade),
            "before-corner" | "before_corner" | "before" | "pre-corner" | "pre_corner" => {
                Some(Self::BeforeCorner)
            }
            _ => None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::AfterCascade => "after-cascade",
            Self::BeforeCorner => "before-corner",
        }
    }

    fn runs_after_cascade(self) -> bool {
        matches!(self, Self::AfterCascade)
    }

    fn runs_before_corner(self) -> bool {
        matches!(self, Self::BeforeCorner)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum PhaseKind {
    SingleTape,
    CrossAxisTapePair,
    ThreeAxisTapeTriplet,
    PairTapeSegments,
    AxisPairTapeSegments,
    AllPairTapeSegments,
    ProtectedCornerArms,
    ProtectedCornerBlock,
    OneFace,
    OppositePair,
    OppositeAndroidPair,
    OppositeAndroidNearPair,
    OppositeAndroidPairRegion,
    AllOppositeAndroidNearPairs,
    BinaryColorSplit,
    OppositeLayerBand,
    OppositeLayerClassBand,
    AdjacentPair,
    CornerTriplet,
}

impl PhaseKind {
    fn defaults() -> Vec<Self> {
        vec![
            Self::OppositePair,
            Self::OppositeAndroidPair,
            Self::AdjacentPair,
            Self::CornerTriplet,
        ]
    }

    fn all() -> Vec<Self> {
        vec![
            Self::SingleTape,
            Self::CrossAxisTapePair,
            Self::ThreeAxisTapeTriplet,
            Self::PairTapeSegments,
            Self::AxisPairTapeSegments,
            Self::AllPairTapeSegments,
            Self::ProtectedCornerArms,
            Self::ProtectedCornerBlock,
            Self::OneFace,
            Self::OppositePair,
            Self::OppositeAndroidPair,
            Self::OppositeAndroidNearPair,
            Self::OppositeAndroidPairRegion,
            Self::AllOppositeAndroidNearPairs,
            Self::BinaryColorSplit,
            Self::OppositeLayerBand,
            Self::OppositeLayerClassBand,
            Self::AdjacentPair,
            Self::CornerTriplet,
        ]
    }

    fn parse(value: &str) -> Option<Vec<Self>> {
        match value.trim().to_ascii_lowercase().as_str() {
            "all" => Some(Self::all()),
            "tape" | "single-tape" | "single_tape" | "one-tape" | "one_tape" => {
                Some(vec![Self::SingleTape])
            }
            "cross-axis-tape-pair"
            | "cross_axis_tape_pair"
            | "alternating-tape-pair"
            | "alternating_tape_pair"
            | "tape-pair"
            | "tape_pair" => Some(vec![Self::CrossAxisTapePair]),
            "three-axis-tape-triplet"
            | "three_axis_tape_triplet"
            | "axis-tape-triplet"
            | "axis_tape_triplet"
            | "tape-triplet"
            | "tape_triplet" => Some(vec![Self::ThreeAxisTapeTriplet]),
            "pair-tape" | "pair_tape" | "pair-tape-segments" | "pair_tape_segments"
            | "single-pair-tape" | "single_pair_tape" => Some(vec![Self::PairTapeSegments]),
            "axis-pair-tapes"
            | "axis_pair_tapes"
            | "axis-pair-tape-segments"
            | "axis_pair_tape_segments" => Some(vec![Self::AxisPairTapeSegments]),
            "all-pair-tapes"
            | "all_pair_tapes"
            | "all-pair-tape-segments"
            | "all_pair_tape_segments"
            | "tape-pair-segment-phase"
            | "tape_pair_segment_phase" => Some(vec![Self::AllPairTapeSegments]),
            "protected-corner-arms"
            | "protected_corner_arms"
            | "protected-corner-9"
            | "protected_corner_9"
            | "corner-9"
            | "corner_9" => Some(vec![Self::ProtectedCornerArms]),
            "protected-corner"
            | "protected_corner"
            | "protected-corner-block"
            | "protected_corner_block"
            | "corner-block"
            | "corner_block" => Some(vec![Self::ProtectedCornerBlock]),
            "one" | "one-face" | "one_face" => Some(vec![Self::OneFace]),
            "opposite" | "opposite-pair" | "opposite_pair" => Some(vec![Self::OppositePair]),
            "opposite-android"
            | "opposite_android"
            | "android-opposite"
            | "android_opposite"
            | "opposite-android-pair"
            | "opposite_android_pair" => Some(vec![Self::OppositeAndroidPair]),
            "near"
            | "near-pair"
            | "near_pair"
            | "opposite-android-near"
            | "opposite_android_near"
            | "opposite-android-near-pair"
            | "opposite_android_near_pair"
            | "near-opposite-android-pair"
            | "near_opposite_android_pair" => Some(vec![Self::OppositeAndroidNearPair]),
            "opposite-android-region"
            | "opposite_android_region"
            | "opposite-android-region-pair"
            | "opposite_android_region_pair"
            | "opposite-android-pair-region"
            | "opposite_android_pair_region"
            | "region-opposite-android-pair"
            | "region_opposite_android_pair" => Some(vec![Self::OppositeAndroidPairRegion]),
            "all-near"
            | "all_near"
            | "all-opposite-android-near"
            | "all_opposite_android_near"
            | "all-opposite-android-near-pairs"
            | "all_opposite_android_near_pairs"
            | "all-android-near"
            | "all_android_near" => Some(vec![Self::AllOppositeAndroidNearPairs]),
            "binary" | "binary-split" | "binary_split" | "binary-color-split"
            | "binary_color_split" | "color-binary" | "color_binary" => {
                Some(vec![Self::BinaryColorSplit])
            }
            "layer"
            | "layer-band"
            | "layer_band"
            | "opposite-layer"
            | "opposite_layer"
            | "opposite-layer-band"
            | "opposite_layer_band" => Some(vec![Self::OppositeLayerBand]),
            "layer-class"
            | "layer_class"
            | "layer-class-band"
            | "layer_class_band"
            | "opposite-layer-class"
            | "opposite_layer_class"
            | "opposite-layer-class-band"
            | "opposite_layer_class_band" => Some(vec![Self::OppositeLayerClassBand]),
            "adjacent" | "adjacent-pair" | "adjacent_pair" => Some(vec![Self::AdjacentPair]),
            "corner" | "corner-triplet" | "corner_triplet" | "triplet" => {
                Some(vec![Self::CornerTriplet])
            }
            _ => None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::SingleTape => "single-tape",
            Self::CrossAxisTapePair => "cross-axis-tape-pair",
            Self::ThreeAxisTapeTriplet => "three-axis-tape-triplet",
            Self::PairTapeSegments => "pair-tape-segments",
            Self::AxisPairTapeSegments => "axis-pair-tape-segments",
            Self::AllPairTapeSegments => "all-pair-tape-segments",
            Self::ProtectedCornerArms => "protected-corner-arms",
            Self::ProtectedCornerBlock => "protected-corner-block",
            Self::OneFace => "one-face",
            Self::OppositePair => "opposite-pair",
            Self::OppositeAndroidPair => "opposite-android-pair",
            Self::OppositeAndroidNearPair => "opposite-android-near-pair",
            Self::OppositeAndroidPairRegion => "opposite-android-region-pair",
            Self::AllOppositeAndroidNearPairs => "all-opposite-android-near-pairs",
            Self::BinaryColorSplit => "binary-color-split",
            Self::OppositeLayerBand => "opposite-layer-band",
            Self::OppositeLayerClassBand => "opposite-layer-class-band",
            Self::AdjacentPair => "adjacent-pair",
            Self::CornerTriplet => "corner-triplet",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PhasePrefixRank {
    Length,
    Global,
    Combined,
    Lookahead,
    PatternDistance,
    PatternLookahead,
    SuffixProbe,
}

impl PhasePrefixRank {
    fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "length" | "len" | "shortest" => Some(Self::Length),
            "global" | "score" | "target-score" | "target_score" => Some(Self::Global),
            "combined" | "phase-global" | "phase_global" => Some(Self::Combined),
            "lookahead" | "probe" | "forward-probe" | "forward_probe" => Some(Self::Lookahead),
            "pattern-distance" | "pattern_distance" | "pdb" | "pdb-distance" | "pdb_distance" => {
                Some(Self::PatternDistance)
            }
            "pattern-lookahead" | "pattern_lookahead" | "pdb-lookahead" | "pdb_lookahead" => {
                Some(Self::PatternLookahead)
            }
            "suffix-probe" | "suffix_probe" | "probe-suffix" | "probe_suffix" => {
                Some(Self::SuffixProbe)
            }
            _ => None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Length => "length",
            Self::Global => "global",
            Self::Combined => "combined",
            Self::Lookahead => "lookahead",
            Self::PatternDistance => "pattern-distance",
            Self::PatternLookahead => "pattern-lookahead",
            Self::SuffixProbe => "suffix-probe",
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Tier {
    max_depth: usize,
    width: usize,
    restarts: usize,
}

#[derive(Debug, Clone)]
struct SolverConfig {
    target_mode: TargetMode,
    table_depth: Option<usize>,
    forward_depth: Option<usize>,
    tiers: Vec<Tier>,
    path_penalty: i32,
    beam_rank: BeamRankMode,
    corridor_diversity_enabled: bool,
    corridor_prefix_len: usize,
    corridor_quota_percent: usize,
    local_window: usize,
    local_depth: usize,
    hit_patience: usize,
    hit_restart_patience: usize,
    retrograde_suffix_beam_first_hit: bool,
    portfolio_first_result: bool,
    operation_profile: OperationProfile,
    operation_portfolio_enabled: bool,
    operation_portfolio_time_limit_ms: u64,
    operation_portfolio_threshold: usize,
    pattern_db_enabled: bool,
    f_pattern_db_portfolio_enabled: bool,
    pattern_db_projection: ProjectionKind,
    pattern_db_depth: usize,
    pattern_db_weight: i32,
    pattern_db_threshold: usize,
    landmark_rescue_enabled: bool,
    landmark_depth: usize,
    landmark_width: usize,
    landmark_candidates: usize,
    landmark_time_limit_ms: u64,
    landmark_suffix_time_limit_ms: u64,
    hard_rescue_enabled: bool,
    hard_rescue_time_limit_ms: u64,
    hard_rescue_tier: Tier,
    pair_region_rescue_enabled: bool,
    pair_region_table_depth: usize,
    pair_region_forward_depth: usize,
    pair_region_time_limit_ms: u64,
    pair_region_suffix_time_limit_ms: u64,
    pair_region_tier: Tier,
    pair_region_prefixes: usize,
    pair_region_preserve_suffix: bool,
    region_pair_weight: i32,
    axis_ring_profile_weight: i32,
    axis_ring_order_weight: i32,
    target_expand_depth: usize,
    max_nodes: u64,
    time_limit_ms: u64,
    rescue_enabled: bool,
    rescue_threshold: usize,
    rescue_time_limit_ms: u64,
    optimize: bool,
}

impl SolverConfig {
    fn table_depth_for(&self, layout: LayoutId) -> usize {
        self.table_depth
            .unwrap_or_else(|| tuned_table_depth(layout, self.target_mode))
    }

    fn forward_depth_for(&self, layout: LayoutId) -> usize {
        self.forward_depth
            .unwrap_or_else(|| tuned_forward_depth(layout, self.target_mode))
    }
}

fn tuned_table_depth(layout: LayoutId, target_mode: TargetMode) -> usize {
    let base = layout.default_table_depth();
    match (layout, target_mode) {
        (LayoutId::F, TargetMode::Android) => base + 1,
        _ => base,
    }
}

fn tuned_forward_depth(layout: LayoutId, target_mode: TargetMode) -> usize {
    let base = layout.default_forward_depth();
    match (layout, target_mode) {
        (LayoutId::F, TargetMode::Android) => base + 1,
        _ => base,
    }
}

#[derive(Debug, Clone)]
struct SolveResult {
    found: bool,
    method: SolveMethod,
    target_used: Option<TargetMode>,
    operation_profile_used: Option<OperationProfile>,
    reason: String,
    raw_moves: Vec<MoveIndex>,
    optimized_moves: Vec<MoveIndex>,
    nodes: u64,
    elapsed_ms: u128,
    first_table_hit: Option<TableHitTelemetry>,
}

#[derive(Debug, Clone)]
struct TableHitTelemetry {
    target: TargetMode,
    operation_profile: OperationProfile,
    beam_rank: BeamRankMode,
    pattern_db: bool,
    depth: usize,
    restart: usize,
    nodes: u64,
    elapsed_ms: u128,
    prefix_len: usize,
    suffix_len: usize,
    total_len: usize,
}

#[derive(Debug, Clone)]
struct SolverArtifacts {
    variants: Vec<SolverVariantArtifacts>,
    pair_region: Option<PairRegionArtifacts>,
    build_nodes: u64,
    build_ms: u128,
}

#[derive(Debug, Clone)]
struct SolverVariantArtifacts {
    target_mode: TargetMode,
    table: HashMap<Key, PathBits>,
    operation_sets: Vec<OperationSetArtifacts>,
    rescue_operations: Option<Vec<Operation>>,
    pattern_db: Option<PatternDb>,
    build_nodes: u64,
    build_ms: u128,
}

#[derive(Debug, Clone)]
struct OperationSetArtifacts {
    profile: OperationProfile,
    operations: Vec<Operation>,
    time_limit_ms: u64,
}

#[derive(Debug, Clone)]
struct PairRegionArtifacts {
    table: HashMap<Key, PathBits>,
    operations: Vec<Operation>,
    preserving_operations: Vec<Operation>,
    build_nodes: u64,
    build_ms: u128,
}

#[derive(Debug, Clone)]
struct PatternDb {
    distances: PatternDbDistances,
    projection: ProjectionKind,
    use_canonical_fallback: bool,
    depth: usize,
    weight: i32,
}

#[derive(Debug, Clone)]
enum PatternDbDistances {
    Face {
        distances: HashMap<PatternKey, u8>,
        canonical_distances: HashMap<PatternKey, u8>,
    },
    Projection {
        distances: HashMap<Vec<u8>, u8>,
    },
}

impl PatternDbDistances {
    fn new(projection: ProjectionKind) -> Self {
        if projection == ProjectionKind::FaceHistogram {
            Self::Face {
                distances: HashMap::new(),
                canonical_distances: HashMap::new(),
            }
        } else {
            Self::Projection {
                distances: HashMap::new(),
            }
        }
    }

    fn record(
        &mut self,
        puzzle: &Puzzle,
        colors: &[Color],
        projection: ProjectionKind,
        distance: u8,
    ) {
        match self {
            Self::Face {
                distances,
                canonical_distances,
            } => {
                distances
                    .entry(face_histogram_key(colors, puzzle))
                    .or_insert(distance);
                canonical_distances
                    .entry(canonical_face_histogram_key(colors, puzzle))
                    .or_insert(distance);
            }
            Self::Projection { distances } => {
                distances
                    .entry(projection_key(puzzle, colors, projection))
                    .or_insert(distance);
            }
        }
    }

    fn distance(
        &self,
        puzzle: &Puzzle,
        colors: &[Color],
        projection: ProjectionKind,
        use_canonical_fallback: bool,
    ) -> Option<u8> {
        match self {
            Self::Face {
                distances,
                canonical_distances,
            } => {
                let direct = distances.get(&face_histogram_key(colors, puzzle));
                if use_canonical_fallback {
                    direct
                        .or_else(|| {
                            canonical_distances.get(&canonical_face_histogram_key(colors, puzzle))
                        })
                        .copied()
                } else {
                    direct.copied()
                }
            }
            Self::Projection { distances } => distances
                .get(&projection_key(puzzle, colors, projection))
                .copied(),
        }
    }

    fn direct_len(&self) -> usize {
        match self {
            Self::Face { distances, .. } => distances.len(),
            Self::Projection { distances } => distances.len(),
        }
    }

    fn canonical_len(&self) -> usize {
        match self {
            Self::Face {
                canonical_distances,
                ..
            } => canonical_distances.len(),
            Self::Projection { .. } => 0,
        }
    }
}

impl PatternDb {
    fn score_state(&self, colors: &[Color], puzzle: &Puzzle) -> i32 {
        self.distance(colors, puzzle)
            .map(i32::from)
            .unwrap_or_else(|| self.depth as i32 + 1)
            * self.weight
    }

    fn distance(&self, colors: &[Color], puzzle: &Puzzle) -> Option<u8> {
        self.distances
            .distance(puzzle, colors, self.projection, self.use_canonical_fallback)
    }

    fn direct_len(&self) -> usize {
        self.distances.direct_len()
    }

    fn canonical_len(&self) -> usize {
        self.distances.canonical_len()
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct PathBits {
    bits: u64,
    len: u8,
}

impl PathBits {
    fn append(self, move_index: usize) -> Self {
        if self.len >= 12 {
            return self;
        }
        Self {
            bits: self.bits | ((move_index as u64) << (self.len as u64 * 5)),
            len: self.len + 1,
        }
    }

    fn prepend(self, move_index: usize) -> Self {
        if self.len >= 12 {
            return self;
        }
        Self {
            bits: (self.bits << 5) | move_index as u64,
            len: self.len + 1,
        }
    }

    fn to_vec(self) -> Vec<MoveIndex> {
        let mut moves = Vec::with_capacity(self.len as usize);
        for index in 0..self.len {
            moves.push(((self.bits >> (index as u64 * 5)) & 31) as MoveIndex);
        }
        moves
    }
}

#[derive(Debug, Clone)]
struct TableEntry {
    colors: Vec<Color>,
    path: PathBits,
    last_move: Option<MoveIndex>,
}

#[derive(Debug, Clone)]
struct ExactEntry {
    colors: Vec<Color>,
    path: PathBits,
    last_move: Option<MoveIndex>,
}

#[derive(Debug, Clone)]
struct PatternEntry {
    colors: Vec<Color>,
    last_move: Option<MoveIndex>,
}

#[derive(Debug, Clone)]
struct Operation {
    moves: Vec<MoveIndex>,
    path: Vec<MoveIndex>,
    is_raw: bool,
    last_move: MoveIndex,
}

#[derive(Debug, Clone)]
struct BeamEntry {
    colors: Vec<Color>,
    path: Vec<MoveIndex>,
    last_move: Option<MoveIndex>,
    score: i32,
    rank_score: i32,
}

#[derive(Debug, Clone)]
struct LandmarkCandidate {
    colors: Vec<Color>,
    path: Vec<MoveIndex>,
    pattern_distance: u8,
    score: i32,
}

#[derive(Debug, Clone)]
struct Limits {
    started: Instant,
    max_nodes: u64,
    time_limit: Duration,
    stop_reason: Option<String>,
}

impl Limits {
    fn new(max_nodes: u64, time_limit_ms: u64) -> Self {
        Self {
            started: Instant::now(),
            max_nodes,
            time_limit: Duration::from_millis(time_limit_ms),
            stop_reason: None,
        }
    }

    fn exceeded(&mut self, nodes: u64) -> bool {
        if nodes >= self.max_nodes {
            self.stop_reason = Some("max_nodes".to_string());
            return true;
        }
        if self.started.elapsed() >= self.time_limit {
            self.stop_reason = Some("timeout".to_string());
            return true;
        }
        false
    }
}

#[derive(Debug, Clone)]
struct BenchOptions {
    layouts: Vec<LayoutId>,
    difficulties: Vec<Difficulty>,
    scramble_lengths: Vec<usize>,
    iterations: usize,
    iteration_list: Option<Vec<usize>>,
    iteration_start: usize,
    seed: u64,
    scramble_profile: ScrambleProfile,
    out_dir: PathBuf,
    avoid_same_tape: bool,
    include_scripts: bool,
    projection_kinds: Vec<ProjectionKind>,
    phase_kinds: Vec<PhaseKind>,
    phase_prefixes: usize,
    phase_prefix_offset: usize,
    phase_probe_prefixes: usize,
    phase_prefix_max_over_min: Option<usize>,
    phase_prefix_rank: PhasePrefixRank,
    phase_rank_lookahead_depth: usize,
    phase_rank_lookahead_width: usize,
    phase_prefix_suffix_probe_time_limit_ms: u64,
    phase_prefix_suffix_probe_candidates: usize,
    phase_tier: Tier,
    phase_time_limit_ms: u64,
    phase_suffix_time_limit_ms: u64,
    phase_profile_portfolio: bool,
    phase_near_misses: usize,
    phase_spec_filters: Vec<String>,
    phase_direct_threshold: usize,
    phase_stop_after_gain: Option<usize>,
    phase_skip_direct: bool,
    phase_corner_shielded: bool,
    phase_corner_shielded_body_depth: usize,
    phase_corner_seed_branches: usize,
    phase_corner_arm_branches: usize,
    phase_corner_pool_specs: bool,
    phase_suffix_hard_rescue: bool,
    macro_shifts: Vec<usize>,
    macro_table_depth: Option<usize>,
    macro_pattern_depth: usize,
    macro_require_suffix: bool,
    commutator_max_len: usize,
    commutator_top: usize,
    commutator_greedy_steps: usize,
    commutator_dynamic_target: bool,
    commutator_plateau_lookahead: usize,
    commutator_suffix_rescue: bool,
    commutator_suffix_time_limit_ms: u64,
    commutator_endgame_depth: usize,
    commutator_endgame_width: usize,
    commutator_endgame_time_limit_ms: u64,
    axis_ring_rescue_enabled: bool,
    axis_ring_rescue_threshold: usize,
    axis_ring_rescue_time_limit_ms: u64,
    axis_ring_rescue_table_depth: usize,
    axis_ring_rescue_pattern_depth: usize,
    axis_ring_rescue_expand_depth: usize,
    axis_ring_rescue_tier: Tier,
    axis_ring_rescue_position: AxisRingRescuePosition,
    axis_ring_rescue_corner_skip_threshold: usize,
    pdb_seed_source: PdbSeedSource,
    pdb_seed_count: usize,
    pdb_seed_step_start: usize,
    pdb_seed_step_end: usize,
    pdb_random_walk_min: usize,
    pdb_random_walk_max: usize,
    feature_repeats: usize,
    e_classic_cascade: bool,
    solver: SolverConfig,
}

#[derive(Debug, Clone)]
struct RunRecord {
    layout: LayoutId,
    difficulty: Difficulty,
    scramble_len: usize,
    iteration: usize,
    seed: u64,
    ariadne_solution_len: usize,
    found: bool,
    reason: String,
    method: SolveMethod,
    raw_solution_len: usize,
    optimized_solution_len: usize,
    optimized_changed: bool,
    target_used: String,
    operation_profile_used: String,
    uniform_solved: bool,
    android_solved: bool,
    nodes: u64,
    elapsed_ms: u128,
    first_table_hit: bool,
    first_table_hit_target: String,
    first_table_hit_profile: String,
    first_table_hit_rank: String,
    first_table_hit_pattern_db: bool,
    first_table_hit_depth: usize,
    first_table_hit_restart: usize,
    first_table_hit_nodes: u64,
    first_table_hit_elapsed_ms: u128,
    first_table_hit_prefix_len: usize,
    first_table_hit_suffix_len: usize,
    first_table_hit_total_len: usize,
    gain_vs_scramble: isize,
    ratio_vs_scramble: f64,
    scramble_script: String,
    raw_solution_script: String,
    optimized_solution_script: String,
}

#[derive(Debug, Clone)]
struct PhaseLabRecord {
    layout: LayoutId,
    difficulty: Difficulty,
    scramble_len: usize,
    iteration: usize,
    seed: u64,
    ariadne_solution_len: usize,
    phase_kind: String,
    phase_spec: String,
    direct_found: bool,
    direct_opt_len: usize,
    direct_elapsed_ms: u128,
    phase_found: bool,
    suffix_found: bool,
    total_found: bool,
    prefix_len: usize,
    suffix_len: usize,
    total_raw_len: usize,
    total_opt_len: usize,
    delta_opt_vs_direct: isize,
    phase_elapsed_ms: u128,
    suffix_elapsed_ms: u128,
    total_elapsed_ms: u128,
    nodes: u64,
    prefixes_available: usize,
    prefixes_tested: usize,
    candidate_min_prefix_len: usize,
    candidate_prefix_lens: String,
    candidate_signatures: String,
    reason: String,
}

#[derive(Debug, Clone)]
struct AriadneCheckRecord {
    layout: LayoutId,
    difficulty: Difficulty,
    scramble_len: usize,
    iteration: usize,
    seed: u64,
    ariadne_stack_len: usize,
    ariadne_unit_len: usize,
    exact_solved: bool,
    uniform_solved: bool,
    android_solved: bool,
    scramble_script: String,
    ariadne_stack_script: String,
    ariadne_solution_script: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AriadneStep {
    axis: u8,
    layer: usize,
    direction: i32,
}

#[derive(Debug, Clone)]
struct PhaseSpec {
    kind: PhaseKind,
    faces: Vec<Face>,
    tapes: Vec<TapeCoord>,
    corner_plan: Option<CornerPlan>,
    label: String,
    near_misses_per_face: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct TapeCoord {
    axis: u8,
    layer: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CornerStage {
    Seed,
    Arms,
    Block,
}

#[derive(Debug, Clone)]
struct CornerPlan {
    stage: CornerStage,
    seed_regions: Vec<(Face, Vec<usize>)>,
    arm_regions: Vec<(Face, Vec<usize>)>,
    block_regions: Vec<(Face, Vec<usize>)>,
    forbidden_tapes: Vec<TapeCoord>,
}

#[derive(Debug, Clone, Default)]
struct GroupStats {
    total: usize,
    found: usize,
    uniform_ok: usize,
    android_ok: usize,
    raw_lens: Vec<usize>,
    opt_lens: Vec<usize>,
    gains: Vec<isize>,
    ratios: Vec<f64>,
    times: Vec<u128>,
    nodes: Vec<u64>,
    first_hit_depths: Vec<usize>,
    first_hit_times: Vec<u128>,
    first_hit_suffix_lens: Vec<usize>,
    methods: BTreeMap<String, usize>,
    target_used: BTreeMap<String, usize>,
    operation_profiles: BTreeMap<String, usize>,
    reasons: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Default)]
struct AriadneCheckGroupStats {
    total: usize,
    exact_ok: usize,
    uniform_ok: usize,
    android_ok: usize,
    stack_lens: Vec<usize>,
    unit_lens: Vec<usize>,
}

#[derive(Debug, Clone, Default)]
struct PhaseGroupStats {
    total: usize,
    phase_found: usize,
    total_found: usize,
    better: usize,
    equal: usize,
    worse: usize,
    direct_lens: Vec<usize>,
    total_lens: Vec<usize>,
    deltas: Vec<isize>,
    portfolio_lens: Vec<usize>,
    portfolio_deltas: Vec<isize>,
    prefix_lens: Vec<usize>,
    suffix_lens: Vec<usize>,
    times: Vec<u128>,
    nodes: Vec<u64>,
    prefixes_available: Vec<usize>,
    prefixes_tested: Vec<usize>,
    candidate_min_prefix_lens: Vec<usize>,
    reasons: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Default)]
struct AxisRingPdbGroupStats {
    total: usize,
    start_hits: usize,
    first_hits: usize,
    path_hit_counts: Vec<usize>,
    ariadne_lens: Vec<usize>,
    first_remaining: Vec<usize>,
    best_remaining: Vec<usize>,
    best_prefix_to_axis: Vec<usize>,
    estimated_totals: Vec<usize>,
    path_distance_remaining_pearsons: Vec<f64>,
}

#[derive(Debug, Clone)]
struct MoveDeltaAuditRecord {
    layout: LayoutId,
    difficulty: Difficulty,
    target: TargetMode,
    operation_profile: OperationProfile,
    scramble_len: usize,
    iteration: usize,
    seed: u64,
    ariadne_unit_len: usize,
    step: usize,
    step_bucket: &'static str,
    remaining_to_solved: usize,
    ariadne_move: String,
    feature: &'static str,
    candidate_count: usize,
    first_match_count: usize,
    raw_rank: f64,
    raw_percentile: f64,
    raw_delta: f64,
    first_match_rank: f64,
    first_match_percentile: f64,
    first_match_delta: f64,
    best_delta: f64,
    worst_delta: f64,
}

#[derive(Debug, Clone, Default)]
struct MoveDeltaAuditGroupStats {
    total: usize,
    candidate_counts: Vec<usize>,
    first_match_counts: Vec<usize>,
    raw_percentiles: Vec<f64>,
    first_match_percentiles: Vec<f64>,
}

#[derive(Debug, Clone)]
struct FeatureCostRecord {
    layout: LayoutId,
    difficulty: Difficulty,
    scramble_len: usize,
    states: usize,
    repeats: usize,
    feature: &'static str,
    evals: usize,
    total_ns: u128,
    mean_ns: f64,
    checksum: u64,
}

#[derive(Debug, Clone)]
struct CommutatorScanRecord {
    layout: LayoutId,
    difficulty: Difficulty,
    max_len: usize,
    sequences: usize,
    pairs_examined: u64,
    unique_permutations: usize,
    identity_count: usize,
    min_support: usize,
    clean_3_cycles: usize,
    double_transpositions: usize,
    histogram: BTreeMap<String, usize>,
    top: Vec<CommutatorCandidate>,
    catalog: Vec<CommutatorCandidate>,
    elapsed_ms: u128,
    truncated: bool,
    reason: String,
}

#[derive(Debug, Clone)]
struct CommutatorCandidate {
    support: usize,
    cycle_type: String,
    cycle_lengths: Vec<usize>,
    a: Vec<MoveIndex>,
    b: Vec<MoveIndex>,
    moves: Vec<MoveIndex>,
}

#[derive(Debug, Clone)]
struct CommutatorApplicabilityRecord {
    layout: LayoutId,
    difficulty: Difficulty,
    target_mode: TargetMode,
    scramble_len: usize,
    iteration: usize,
    seed: u64,
    initial_mismatches: usize,
    clean_3_catalog_size: usize,
    distinct_triples: usize,
    covered_positions: usize,
    improving_commutators: usize,
    strong_commutators: usize,
    best_delta: isize,
    best_after_mismatches: usize,
    best_direction: String,
    best_commutator: String,
}

#[derive(Debug, Clone)]
struct CommutatorGreedyRecord {
    layout: LayoutId,
    difficulty: Difficulty,
    target_mode: TargetMode,
    scramble_len: usize,
    iteration: usize,
    seed: u64,
    ariadne_solution_len: usize,
    initial_mismatches: usize,
    final_mismatches: usize,
    found: bool,
    reason: String,
    commutator_steps: usize,
    greedy_raw_len: usize,
    greedy_optimized_len: usize,
    suffix_attempted: bool,
    suffix_found: bool,
    suffix_raw_len: usize,
    suffix_optimized_len: usize,
    suffix_reason: String,
    raw_solution_len: usize,
    optimized_solution_len: usize,
    best_step_delta: isize,
    mean_step_delta: f64,
    elapsed_ms: u128,
    raw_solution_script: String,
    optimized_solution_script: String,
}

#[derive(Debug, Clone)]
struct CommutatorPlateauRecord {
    layout: LayoutId,
    difficulty: Difficulty,
    target_mode: TargetMode,
    scramble_len: usize,
    iteration: usize,
    seed: u64,
    greedy_reason: String,
    greedy_steps: usize,
    greedy_raw_len: usize,
    initial_mismatches: usize,
    plateau_mismatches: usize,
    best_dynamic_mismatches: usize,
    residual_canonical_parity: String,
    residual_flex_parity: String,
    full_canonical_parity: String,
    support_touch_1: usize,
    support_touch_2: usize,
    support_touch_3: usize,
    support_touch_4: usize,
    support_subset_mismatch: usize,
    support_contains_all_mismatch: usize,
    direct_improving: usize,
    direct_nonworsening: usize,
    direct_best_delta: isize,
    direct_best_after: usize,
    mismatch_positions: String,
    mismatch_details: String,
    transition_counts: String,
}

#[derive(Debug, Clone)]
struct CommutatorPrimitive {
    moves: Vec<MoveIndex>,
    inverse_moves: Vec<MoveIndex>,
    positions: Vec<usize>,
}

#[derive(Debug, Clone)]
struct GreedyCommutatorSolve {
    initial_mismatches: usize,
    steps: usize,
    moves: Vec<MoveIndex>,
    deltas: Vec<isize>,
    reason: String,
}

#[derive(Debug, Clone)]
struct CommutatorEndgameRecord {
    layout: LayoutId,
    difficulty: Difficulty,
    target_mode: TargetMode,
    scramble_len: usize,
    iteration: usize,
    seed: u64,
    ariadne_solution_len: usize,
    greedy_reason: String,
    greedy_steps: usize,
    greedy_raw_len: usize,
    plateau_mismatches: usize,
    found: bool,
    reason: String,
    endgame_found: bool,
    endgame_depth: usize,
    endgame_nodes: u64,
    endgame_raw_len: usize,
    endgame_best_mismatches: usize,
    total_raw_len: usize,
    total_optimized_len: usize,
    final_mismatches: usize,
    elapsed_ms: u128,
    mismatch_positions: String,
    endgame_script: String,
    optimized_script: String,
}

#[derive(Debug, Clone)]
struct CommutatorBranchAuditRecord {
    layout: LayoutId,
    difficulty: Difficulty,
    target_mode: TargetMode,
    scramble_len: usize,
    iteration: usize,
    seed: u64,
    source: String,
    gate: usize,
    step: usize,
    ariadne_remaining: usize,
    mismatches: usize,
    filtered_primitives: usize,
    filtered_directions: usize,
    capped_primitives: usize,
    touch_1: usize,
    touch_2: usize,
    touch_3: usize,
    touch_4_or_more: usize,
}

#[derive(Debug, Clone)]
struct CommutatorDecompositionRecord {
    layout: LayoutId,
    difficulty: Difficulty,
    target_mode: TargetMode,
    scramble_len: usize,
    iteration: usize,
    seed: u64,
    ariadne_solution_len: usize,
    plateau_mismatches: usize,
    plateau_positions: String,
    exact_support_available: bool,
    subset_support_count: usize,
    contains_all_support_count: usize,
    target_min_mismatches: usize,
    target_max_mismatches: usize,
    target_min_count: usize,
    direct_found: bool,
    direct_reason: String,
    direct_steps: usize,
    direct_raw_len: usize,
    direct_opt_len: usize,
    direct_total_opt_len: usize,
    direct_final_mismatches: usize,
    unit_found: bool,
    unit_reason: String,
    unit_raw_len: usize,
    unit_opt_len: usize,
    unit_total_opt_len: usize,
    unit_final_mismatches: usize,
    unit_nodes: u64,
    unit_elapsed_ms: u128,
    helper_found: bool,
    helper_reason: String,
    helper_steps: usize,
    helper_raw_len: usize,
    helper_opt_len: usize,
    helper_total_opt_len: usize,
    helper_final_mismatches: usize,
    helper_nodes: u64,
    helper_elapsed_ms: u128,
    helper_endgame_found: bool,
    helper_endgame_reason: String,
    helper_endgame_raw_len: usize,
    helper_endgame_opt_len: usize,
    helper_endgame_total_opt_len: usize,
    helper_endgame_final_mismatches: usize,
    helper_endgame_nodes: u64,
    helper_endgame_elapsed_ms: u128,
}

#[derive(Debug, Clone)]
struct RingResidueAuditRecord {
    layout: LayoutId,
    difficulty: Difficulty,
    target_mode: TargetMode,
    scramble_len: usize,
    iteration: usize,
    seed: u64,
    ariadne_solution_len: usize,
    gate: usize,
    prefix_found: bool,
    prefix_reason: String,
    prefix_rank: String,
    prefix_profile: String,
    prefix_len: usize,
    prefix_mismatches: usize,
    prefix_nodes: u64,
    prefix_elapsed_ms: u128,
    tail_found: bool,
    tail_reason: String,
    tail_raw_len: usize,
    tail_opt_len: usize,
    tail_mismatches: usize,
    tail_nodes: u64,
    tail_elapsed_ms: u128,
    total_found: bool,
    total_opt_len: usize,
    final_mismatches: usize,
}

#[derive(Debug, Clone)]
struct LastKRepairRecord {
    layout: LayoutId,
    difficulty: Difficulty,
    target_mode: TargetMode,
    scramble_len: usize,
    iteration: usize,
    seed: u64,
    ariadne_solution_len: usize,
    source: String,
    table_depth: usize,
    table_states: usize,
    table_gate_counts: String,
    table_depth_counts: String,
    table_nodes: u64,
    table_elapsed_ms: u128,
    table_truncated: bool,
    table_reason: String,
    source_mismatches: usize,
    source_prefix_len: usize,
    source_reason: String,
    hit: bool,
    hit_suffix_len: usize,
    total_found: bool,
    total_opt_len: usize,
    final_mismatches: usize,
}

#[derive(Debug, Clone)]
struct LastKRepairTable {
    repair: HashMap<Key, PathBits>,
    gate_counts: Vec<usize>,
    depth_counts: Vec<usize>,
    nodes: u64,
    elapsed_ms: u128,
    truncated: bool,
    reason: String,
}

#[derive(Debug, Clone)]
struct ExactShortcutRecord {
    layout: LayoutId,
    difficulty: Difficulty,
    target_mode: TargetMode,
    scramble_len: usize,
    iteration: usize,
    seed: u64,
    ariadne_solution_len: usize,
    table_depth: usize,
    forward_depth: usize,
    proof_bound: usize,
    table_states: usize,
    table_nodes: u64,
    table_elapsed_ms: u128,
    found: bool,
    proved_optimal: bool,
    optimal_len: usize,
    shortcut_moves: isize,
    shortcut_ratio: f64,
    significant_shortcut: bool,
    search_nodes: u64,
    search_elapsed_ms: u128,
    forward_seen: usize,
    table_hits: usize,
    complete: bool,
    reason: String,
    scramble_script: String,
    optimal_script: String,
}

#[derive(Debug, Clone)]
struct ExactShortcutSearchResult {
    found: bool,
    moves: Vec<MoveIndex>,
    nodes: u64,
    elapsed_ms: u128,
    forward_seen: usize,
    table_hits: usize,
    complete: bool,
    reason: String,
}

#[derive(Debug, Clone)]
struct CommutatorEndgameSearchResult {
    found: bool,
    reason: String,
    moves: Vec<MoveIndex>,
    depth: usize,
    nodes: u64,
    best_mismatches: usize,
}

#[derive(Debug, Clone)]
struct ResiduePrefixResult {
    found: bool,
    reason: String,
    rank_label: String,
    profile_label: String,
    colors: Vec<Color>,
    moves: Vec<MoveIndex>,
    mismatches: usize,
    nodes: u64,
    elapsed_ms: u128,
}

#[derive(Debug, Clone)]
struct CommutatorHelperTailResult {
    found: bool,
    reason: String,
    moves: Vec<MoveIndex>,
    steps: usize,
    nodes: u64,
}

#[derive(Debug, Clone)]
struct CommutatorEndgameEntry {
    colors: Vec<Color>,
    moves: Vec<MoveIndex>,
    mismatches: usize,
    score: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CommutatorEndgameRank {
    mismatches: usize,
    score: i32,
    unit_len: usize,
}

impl Ord for CommutatorEndgameRank {
    fn cmp(&self, other: &Self) -> Ordering {
        self.mismatches
            .cmp(&other.mismatches)
            .then_with(|| self.score.cmp(&other.score))
            .then_with(|| self.unit_len.cmp(&other.unit_len))
    }
}

impl PartialOrd for CommutatorEndgameRank {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone)]
struct CommutatorEndgameHeapItem {
    rank: CommutatorEndgameRank,
    serial: usize,
    entry: CommutatorEndgameEntry,
}

impl PartialEq for CommutatorEndgameHeapItem {
    fn eq(&self, other: &Self) -> bool {
        self.rank == other.rank && self.serial == other.serial
    }
}

impl Eq for CommutatorEndgameHeapItem {}

impl Ord for CommutatorEndgameHeapItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.rank
            .cmp(&other.rank)
            .then_with(|| self.serial.cmp(&other.serial))
    }
}

impl PartialOrd for CommutatorEndgameHeapItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone)]
struct PlateauRepairCandidate {
    after_mismatches: usize,
    moves: Vec<MoveIndex>,
    colors: Vec<Color>,
}

#[derive(Debug, Clone)]
struct BeamDirectionSurvivalRecord {
    layout: LayoutId,
    difficulty: Difficulty,
    target: TargetMode,
    operation_profile: OperationProfile,
    scramble_len: usize,
    iteration: usize,
    seed: u64,
    ariadne_unit_len: usize,
    ariadne_first_move: String,
    beam_width: usize,
    depth: usize,
    survival: bool,
    extinction_layer: Option<usize>,
    direction_count: usize,
    target_direction_entries: usize,
    max_direction_entries: usize,
    beam_size: usize,
    nodes: u64,
}

#[derive(Debug, Clone, Default)]
struct BeamDirectionSurvivalGroupStats {
    total: usize,
    alive: usize,
    direction_counts: Vec<usize>,
    target_direction_entries: Vec<usize>,
    max_direction_entries: Vec<usize>,
    beam_sizes: Vec<usize>,
    nodes: Vec<u64>,
}

#[derive(Debug, Clone, Default)]
struct BeamDirectionExtinctionStats {
    total: usize,
    extinct: usize,
    extinct_by_2: usize,
    extinct_by_5: usize,
    extinction_layers: Vec<usize>,
}

#[derive(Debug, Clone)]
struct BeamPrefixSurvivalRecord {
    layout: LayoutId,
    difficulty: Difficulty,
    target: TargetMode,
    operation_profile: OperationProfile,
    scramble_len: usize,
    iteration: usize,
    seed: u64,
    ariadne_unit_len: usize,
    beam_width: usize,
    depth: usize,
    target_prefix_len: usize,
    path_prefix_alive: bool,
    prefix_state_alive: bool,
    max_matching_prefix: usize,
    matching_states_count: usize,
    strict_path_state_count: usize,
    max_prefix_entries: usize,
    mean_matching_prefix: f64,
    beam_size: usize,
    nodes: u64,
}

#[derive(Debug, Clone, Default)]
struct BeamPrefixSurvivalGroupStats {
    total: usize,
    path_prefix_alive: usize,
    prefix_state_alive: usize,
    max_matching_prefixes: Vec<usize>,
    matching_states_counts: Vec<usize>,
    strict_path_state_counts: Vec<usize>,
    max_prefix_entries: Vec<usize>,
    mean_matching_prefixes: Vec<f64>,
    beam_sizes: Vec<usize>,
    nodes: Vec<u64>,
}

#[derive(Debug, Clone)]
struct BackwardMidpointAuditRecord {
    layout: LayoutId,
    difficulty: Difficulty,
    target: TargetMode,
    operation_profile: OperationProfile,
    scramble_len: usize,
    iteration: usize,
    seed: u64,
    ariadne_unit_len: usize,
    midpoint_step: usize,
    beam_width: usize,
    depth: usize,
    target_states: usize,
    operation_count: usize,
    candidate_hit: bool,
    selected_hit: bool,
    final_frontier_hit: bool,
    first_candidate_layer: Option<usize>,
    first_selected_layer: Option<usize>,
    best_candidate_layer: Option<usize>,
    best_candidate_rank: Option<usize>,
    best_candidate_count: Option<usize>,
    best_candidate_percentile: Option<f64>,
    final_frontier_size: usize,
    target_score: i32,
    final_best_score: Option<i32>,
    final_worst_score: Option<i32>,
    nodes: u64,
    elapsed_ms: u128,
}

#[derive(Debug, Clone, Default)]
struct BackwardMidpointGroupStats {
    total: usize,
    candidate_hits: usize,
    selected_hits: usize,
    final_hits: usize,
    ariadne_lens: Vec<usize>,
    midpoint_steps: Vec<usize>,
    first_candidate_layers: Vec<usize>,
    first_selected_layers: Vec<usize>,
    best_candidate_percentiles: Vec<f64>,
    final_frontier_sizes: Vec<usize>,
    target_scores: Vec<i32>,
    final_best_scores: Vec<i32>,
    final_worst_scores: Vec<i32>,
    nodes: Vec<u64>,
    elapsed_ms: Vec<u128>,
}

#[derive(Debug, Clone)]
struct BackwardMidpointBeamEntry {
    colors: Vec<Color>,
    key: Key,
    path_len: usize,
    last_move: Option<MoveIndex>,
    score: i32,
    rank_score: i32,
}

#[derive(Debug, Clone)]
struct HeuristicAuditRecord {
    layout: LayoutId,
    difficulty: Difficulty,
    target: TargetMode,
    depth: usize,
    states: usize,
    histogram_keys: usize,
    canonical_histogram_keys: usize,
    nodes: u64,
    elapsed_ms: u128,
    pearson_score_distance: f64,
    truncated: bool,
    reason: String,
    depth_counts: Vec<usize>,
    mean_score_by_depth: Vec<f64>,
}

#[derive(Debug, Clone, Copy)]
struct PhaseAuditEntry {
    phase: u64,
    path: PathBits,
}

#[derive(Debug, Clone)]
struct PhaseAuditNode {
    permutation: Vec<u8>,
    colors: Vec<Color>,
    phase: u64,
    path: PathBits,
    last_move: Option<MoveIndex>,
}

#[derive(Debug, Clone)]
struct PhaseAuditConflict {
    key_kind: &'static str,
    existing_phase: u64,
    new_phase: u64,
    existing_path: PathBits,
    new_path: PathBits,
}

#[derive(Debug, Clone)]
struct MacroOp {
    move_index: MoveIndex,
    shift: usize,
}

#[derive(Debug, Clone)]
struct MacroSubgroupNode {
    permutation: Vec<u8>,
    colors: Vec<Color>,
    depth: usize,
    last_macro: Option<usize>,
}

#[derive(Debug, Clone)]
struct MacroSubgroupAuditResult {
    shift: usize,
    macro_count: usize,
    explored: u64,
    permutation_states: usize,
    color_states: usize,
    depth_counts: Vec<usize>,
    color_depth_counts: Vec<usize>,
    exhausted: bool,
    depth_limited: bool,
    truncated: bool,
    elapsed_ms: u128,
}

#[derive(Debug, Clone)]
struct MacroTargetEntry {
    colors: Vec<Color>,
    path: PathBits,
    last_macro: Option<usize>,
}

#[derive(Debug, Clone)]
struct MacroTargetArtifacts {
    table: HashMap<Key, PathBits>,
    target_colors: Vec<Vec<Color>>,
    macro_ops: Vec<MacroOp>,
    depth_counts: Vec<usize>,
    build_nodes: u64,
    build_ms: u128,
    projection_db: Option<PatternDb>,
    projection_build_nodes: u64,
    projection_build_ms: u128,
}

#[derive(Debug, Clone)]
struct MacroTargetHit {
    prefix: Vec<MoveIndex>,
    suffix_macro: PathBits,
    nodes: u64,
    reason: String,
}

#[derive(Debug, Clone)]
struct MacroTargetAttempt {
    hit: Option<MacroTargetHit>,
    nodes: u64,
    reason: String,
}

#[derive(Debug, Clone)]
struct MacroTwoStageRecord {
    layout: LayoutId,
    difficulty: Difficulty,
    scramble_len: usize,
    iteration: usize,
    seed: u64,
    ariadne_solution_len: usize,
    found: bool,
    reason: String,
    prefix_len: usize,
    suffix_macro_len: usize,
    suffix_unit_len: usize,
    raw_solution_len: usize,
    optimized_solution_len: usize,
    android_solved: bool,
    nodes: u64,
    elapsed_ms: u128,
    scramble_script: String,
    raw_solution_script: String,
    optimized_solution_script: String,
}

#[derive(Debug, Clone)]
struct RestrictedTargetEntry {
    colors: Vec<Color>,
    suffix: Vec<MoveIndex>,
    last_move: Option<MoveIndex>,
}

#[derive(Debug, Clone)]
struct RestrictedTargetArtifacts {
    table: HashMap<Key, Vec<MoveIndex>>,
    target_colors: Vec<Vec<Color>>,
    allowed_moves: Vec<MoveIndex>,
    depth_counts: Vec<usize>,
    build_nodes: u64,
    build_ms: u128,
    projection_db: Option<PatternDb>,
    projection_build_nodes: u64,
    projection_build_ms: u128,
    axis_hint: Option<u8>,
    axis_ring_profiles: Vec<AxisRingProfile>,
}

#[derive(Debug, Clone)]
struct AxisRingProfile {
    fixed_colors: (Color, Color),
    ring_counts: Vec<[usize; 6]>,
    ring_sequences: Vec<Vec<Color>>,
}

#[derive(Debug, Clone)]
struct RestrictedTargetHit {
    prefix: Vec<MoveIndex>,
    suffix: Vec<MoveIndex>,
    nodes: u64,
    reason: String,
}

#[derive(Debug, Clone)]
struct RestrictedTargetAttempt {
    hit: Option<RestrictedTargetHit>,
    nodes: u64,
    reason: String,
}

#[derive(Debug, Clone)]
struct AxisRingRecord {
    layout: LayoutId,
    difficulty: Difficulty,
    scramble_len: usize,
    iteration: usize,
    seed: u64,
    axis: u8,
    ariadne_solution_len: usize,
    found: bool,
    reason: String,
    prefix_len: usize,
    suffix_len: usize,
    raw_solution_len: usize,
    optimized_solution_len: usize,
    android_solved: bool,
    nodes: u64,
    elapsed_ms: u128,
    scramble_script: String,
    raw_solution_script: String,
    optimized_solution_script: String,
}

#[derive(Debug, Clone)]
struct AxisRingRescueArtifacts {
    target_mode: TargetMode,
    axes: Vec<(u8, RestrictedTargetArtifacts)>,
    operations: Vec<Operation>,
    operation_profile: OperationProfile,
}

#[derive(Debug, Clone)]
struct AxisRingRescueResult {
    found: bool,
    axis: Option<u8>,
    reason: String,
    prefix_len: usize,
    suffix_len: usize,
    raw_moves: Vec<MoveIndex>,
    optimized_moves: Vec<MoveIndex>,
    nodes: u64,
    elapsed_ms: u128,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProjectionKind {
    FaceHistogram,
    CanonicalFaceHistogram,
    AndroidPairFaces,
    FaceHomeMatches,
    FacePairHomeMatches,
    TapeHistogram,
    TapePairHistogram,
    TapeQuality,
    TapePairQuality,
    TapeSegments,
    TapePairSegments,
}

impl ProjectionKind {
    fn all() -> Vec<Self> {
        vec![
            Self::FaceHistogram,
            Self::CanonicalFaceHistogram,
            Self::AndroidPairFaces,
            Self::FaceHomeMatches,
            Self::FacePairHomeMatches,
            Self::TapeHistogram,
            Self::TapePairHistogram,
            Self::TapeQuality,
            Self::TapePairQuality,
            Self::TapeSegments,
            Self::TapePairSegments,
        ]
    }

    fn label(self) -> &'static str {
        match self {
            Self::FaceHistogram => "face-histogram",
            Self::CanonicalFaceHistogram => "canonical-face-histogram",
            Self::AndroidPairFaces => "android-pair-faces",
            Self::FaceHomeMatches => "face-home-matches",
            Self::FacePairHomeMatches => "face-pair-home-matches",
            Self::TapeHistogram => "tape-histogram",
            Self::TapePairHistogram => "tape-pair-histogram",
            Self::TapeQuality => "tape-quality",
            Self::TapePairQuality => "tape-pair-quality",
            Self::TapeSegments => "tape-segments",
            Self::TapePairSegments => "tape-pair-segments",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "face" | "face-histogram" | "face_histogram" => Some(Self::FaceHistogram),
            "canonical-face"
            | "canonical_face"
            | "canonical-face-histogram"
            | "canonical_face_histogram" => Some(Self::CanonicalFaceHistogram),
            "android-pair-faces" | "android_pair_faces" | "pair-faces" | "pair_faces" => {
                Some(Self::AndroidPairFaces)
            }
            "face-home" | "face_home" | "face-home-matches" | "face_home_matches" | "home" => {
                Some(Self::FaceHomeMatches)
            }
            "face-pair-home"
            | "face_pair_home"
            | "face-pair-home-matches"
            | "face_pair_home_matches"
            | "pair-home" => Some(Self::FacePairHomeMatches),
            "tape-histogram" | "tape_histogram" | "tape" => Some(Self::TapeHistogram),
            "tape-pair-histogram" | "tape_pair_histogram" | "tape-pair" | "tape_pair" => {
                Some(Self::TapePairHistogram)
            }
            "tape-quality" | "tape_quality" => Some(Self::TapeQuality),
            "tape-pair-quality" | "tape_pair_quality" => Some(Self::TapePairQuality),
            "tape-segments" | "tape_segments" => Some(Self::TapeSegments),
            "tape-pair-segments" | "tape_pair_segments" => Some(Self::TapePairSegments),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
struct ProjectionAuditRecord {
    layout: LayoutId,
    difficulty: Difficulty,
    target: TargetMode,
    projection: ProjectionKind,
    depth: usize,
    states: usize,
    keys: usize,
    mean_depth_span: f64,
    p95_depth_span: usize,
    max_depth_span: usize,
    mean_states_per_key: f64,
    p95_states_per_key: usize,
    max_states_per_key: usize,
    nodes: u64,
    elapsed_ms: u128,
    truncated: bool,
    reason: String,
}

#[derive(Debug, Clone)]
struct AxisRingPdbRecord {
    layout: LayoutId,
    difficulty: Difficulty,
    target: TargetMode,
    seed_source: PdbSeedSource,
    scramble_len: usize,
    iteration: usize,
    seed: u64,
    axis_table_depth: usize,
    target_expand_depth: usize,
    pdb_depth: usize,
    target_states: usize,
    pdb_states: usize,
    depth_counts: Vec<usize>,
    build_nodes: u64,
    build_elapsed_ms: u128,
    truncated: bool,
    reason: String,
    ariadne_unit_len: usize,
    path_states: usize,
    hit_count: usize,
    start_hit_distance: Option<u8>,
    first_hit_step: Option<usize>,
    first_hit_distance: Option<u8>,
    first_hit_remaining_to_solved: Option<usize>,
    best_hit_step: Option<usize>,
    best_hit_distance: Option<u8>,
    best_hit_remaining_to_solved: Option<usize>,
    best_prefix_to_axis_ring: Option<usize>,
    max_axis_suffix: usize,
    estimated_total_with_max_suffix: Option<usize>,
    path_distance_remaining_pearson: f64,
}

#[derive(Debug, Clone)]
struct AxisRingPdb {
    distances: HashMap<Key, u8>,
    depth_counts: Vec<usize>,
    target_states: usize,
    max_axis_suffix: usize,
    nodes: u64,
    elapsed_ms: u128,
    truncated: bool,
    reason: String,
}

#[derive(Debug, Clone)]
struct PdbSeedSet {
    states: Vec<Vec<Color>>,
    nodes: u64,
    max_suffix: usize,
    truncated: bool,
    reason: String,
}

#[derive(Debug, Clone)]
struct AxisRingPdbEntry {
    colors: Vec<Color>,
    last_move: Option<MoveIndex>,
}

#[derive(Debug, Clone)]
struct ProjectionBucket {
    count: usize,
    min_distance: usize,
    max_distance: usize,
}

impl ProjectionBucket {
    fn new(distance: usize) -> Self {
        Self {
            count: 1,
            min_distance: distance,
            max_distance: distance,
        }
    }

    fn add(&mut self, distance: usize) {
        self.count += 1;
        self.min_distance = self.min_distance.min(distance);
        self.max_distance = self.max_distance.max(distance);
    }

    fn depth_span(&self) -> usize {
        self.max_distance.saturating_sub(self.min_distance)
    }
}

#[derive(Debug, Clone)]
struct ScoreDistanceStats {
    count: usize,
    sum_score: f64,
    sum_distance: f64,
    sum_score_sq: f64,
    sum_distance_sq: f64,
    sum_score_distance: f64,
}

impl ScoreDistanceStats {
    fn new() -> Self {
        Self {
            count: 0,
            sum_score: 0.0,
            sum_distance: 0.0,
            sum_score_sq: 0.0,
            sum_distance_sq: 0.0,
            sum_score_distance: 0.0,
        }
    }

    fn add(&mut self, score: i32, distance: usize) {
        let score = score as f64;
        let distance = distance as f64;
        self.count += 1;
        self.sum_score += score;
        self.sum_distance += distance;
        self.sum_score_sq += score * score;
        self.sum_distance_sq += distance * distance;
        self.sum_score_distance += score * distance;
    }

    fn pearson(&self) -> f64 {
        if self.count < 2 {
            return 0.0;
        }
        let n = self.count as f64;
        let numerator = n * self.sum_score_distance - self.sum_score * self.sum_distance;
        let score_var = n * self.sum_score_sq - self.sum_score * self.sum_score;
        let distance_var = n * self.sum_distance_sq - self.sum_distance * self.sum_distance;
        let denominator = (score_var * distance_var).sqrt();
        if denominator == 0.0 {
            0.0
        } else {
            numerator / denominator
        }
    }
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    let command = if !args.is_empty() && !args[0].starts_with("--") {
        args.remove(0)
    } else {
        "bench".to_string()
    };

    match command.as_str() {
        "bench" => run_bench(parse_bench_options(args)?),
        "solve-state" | "solve-json" | "web-solve" => run_solve_state(args),
        "ariadne-check" | "ariadne" => run_ariadne_check(parse_bench_options(args)?),
        "heuristic-audit" | "heuristics" | "audit" => {
            run_heuristic_audit(parse_bench_options(args)?)
        }
        "projection-audit" | "projections" | "projection" => {
            run_projection_audit(parse_bench_options(args)?)
        }
        "axis-ring-pdb-audit" | "axis-pdb-audit" | "ring-pdb-audit" => {
            run_axis_ring_pdb_audit(parse_bench_options(args)?)
        }
        "move-delta-audit" | "delta-audit" | "policy-audit" => {
            run_move_delta_audit(parse_bench_options(args)?)
        }
        "feature-cost-audit" | "feature-cost" | "cost-audit" => {
            run_feature_cost_audit(parse_bench_options(args)?)
        }
        "beam-direction-survival-audit" | "direction-survival-audit" | "direction-survival" => {
            run_beam_direction_survival_audit(parse_bench_options(args)?)
        }
        "beam-prefix-survival-audit" | "prefix-survival-audit" | "prefix-survival" => {
            run_beam_prefix_survival_audit(parse_bench_options(args)?)
        }
        "backward-midpoint-audit" | "backward-frontier-audit" | "backward-beam-audit" => {
            run_backward_midpoint_audit(parse_bench_options(args)?)
        }
        "phase-quotient-audit" | "phase-quotient" | "quotient-audit" => {
            run_phase_quotient_audit(parse_bench_options(args)?)
        }
        "macro-subgroup-audit" | "macro-subgroup" | "subgroup-audit" => {
            run_macro_subgroup_audit(parse_bench_options(args)?)
        }
        "shortcut-audit" | "exact-shortcut-audit" | "optimal-shortcut-audit" => {
            run_exact_shortcut_audit(parse_bench_options(args)?)
        }
        "macro6-two-stage" | "macro-two-stage" | "macro6-lab" => {
            run_macro_two_stage(parse_bench_options(args)?)
        }
        "commutator-scan" | "commutators" | "commutator-audit" => {
            run_commutator_scan(parse_bench_options(args)?)
        }
        "commutator-applicability-audit" | "commutator-applicability" | "commutator-coverage" => {
            run_commutator_applicability_audit(parse_bench_options(args)?)
        }
        "commutator-greedy-audit" | "commutator-greedy" | "commutator-reduction-audit" => {
            run_commutator_greedy_audit(parse_bench_options(args)?)
        }
        "commutator-plateau-audit" | "commutator-plateau" | "plateau-residue-audit" => {
            run_commutator_plateau_audit(parse_bench_options(args)?)
        }
        "commutator-endgame-audit" | "commutator-endgame" | "endgame-audit" => {
            run_commutator_endgame_audit(parse_bench_options(args)?)
        }
        "commutator-decomposition-audit" | "decomposition-audit" | "residual-closure-control" => {
            run_commutator_decomposition_audit(parse_bench_options(args)?)
        }
        "ring-prefix-to-residue-audit" | "ring-residue-audit" | "unified-tail-audit" => {
            run_ring_prefix_to_residue_audit(parse_bench_options(args)?)
        }
        "last-k-repair-audit" | "last-k-repair" | "residual-repair-table-audit" => {
            run_last_k_repair_audit(parse_bench_options(args)?)
        }
        "commutator-branch-audit" | "commutator-beam-injection-audit" | "branch-audit" => {
            run_commutator_branch_audit(parse_bench_options(args)?)
        }
        "axis-ring-audit" | "ring-audit" => run_axis_ring_audit(parse_bench_options(args)?),
        "middle-subgroup-audit" | "middle-subgroup" => {
            run_middle_subgroup_audit(parse_bench_options(args)?)
        }
        "axis-ring-two-stage" | "ring-two-stage" => {
            run_axis_ring_two_stage(parse_bench_options(args)?)
        }
        "phase-lab" | "phaselab" | "phase" => run_phase_lab(parse_bench_options(args)?),
        "e-classic-cascade" | "e-cascade" | "cascade" => {
            run_phase_lab(parse_e_classic_cascade_options(args)?)
        }
        "help" | "--help" | "-h" => {
            print_help();
            Ok(())
        }
        other => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("unknown command: {other}"),
        )),
    }
}

fn print_help() {
    println!(
        r#"SkimmIQ native solver benchmark

Usage:
  cargo run --release -- bench [options]
  cargo run --release -- solve-state --layout E --difficulty classic --colors 0,1,2,...
  cargo run --release -- ariadne-check [options]
  cargo run --release -- heuristic-audit [options]
  cargo run --release -- projection-audit [options]
  cargo run --release -- axis-ring-pdb-audit [options]
  cargo run --release -- move-delta-audit [options]
  cargo run --release -- feature-cost-audit [options]
  cargo run --release -- beam-direction-survival-audit [options]
  cargo run --release -- beam-prefix-survival-audit [options]
  cargo run --release -- backward-midpoint-audit [options]
  cargo run --release -- phase-quotient-audit [options]
  cargo run --release -- macro-subgroup-audit [options]
  cargo run --release -- shortcut-audit [options]
  cargo run --release -- macro6-two-stage [options]
  cargo run --release -- commutator-scan [options]
  cargo run --release -- commutator-applicability-audit [options]
  cargo run --release -- commutator-greedy-audit [options]
  cargo run --release -- commutator-plateau-audit [options]
  cargo run --release -- commutator-endgame-audit [options]
  cargo run --release -- commutator-decomposition-audit [options]
  cargo run --release -- ring-prefix-to-residue-audit [options]
  cargo run --release -- last-k-repair-audit [options]
  cargo run --release -- commutator-branch-audit [options]
  cargo run --release -- axis-ring-audit [options]
  cargo run --release -- middle-subgroup-audit [options]
  cargo run --release -- axis-ring-two-stage [options]
  cargo run --release -- phase-lab [options]
  cargo run --release -- e-classic-cascade [options]

Options:
  --layouts A,B,C,D,E,F
  --difficulties easy,moderate,classic
  --scrambles 5,10,20,40
  --scramble-profile lab|android-original
  --iterations 10
  --iteration-list 1,5,9
  --iteration-start 1
  --seed 12345
  --target uniform|android|android-multi|android-portfolio|pair-region
  --table-depth N
  --forward-depth N
  --path-penalty N
  --beam-rank score|pattern-distance|pattern-hybrid|ring-rotation|ring-hybrid|ring-portfolio
  --corridor-diversity
  --corridor-prefix-len N
  --corridor-quota-percent N
  --local-window N
  --local-depth N
  --hit-patience N
  --hit-restarts N
  --retrograde-suffix-beam-first-hit
  --portfolio-first-result
  --retrograde-suffix-early-stop   # enables both switches above
  --operation-profile auto|raw|basic|pairs|conjugates|expanded|expanded-parallel|expanded-wide
  --operation-portfolio-time-limit-ms N
  --operation-portfolio-threshold N
  --pattern-db-depth N
  --pattern-db-weight N
  --pattern-db-threshold N
  --pattern-db-projection face-histogram|tape-quality|tape-pair-segments
  --f-pattern-db-portfolio
  --landmark-depth N
  --landmark-width N
  --landmark-candidates N
  --landmark-time-limit-ms N
  --landmark-suffix-time-limit-ms N
  --hard-rescue-time-limit-ms N
  --hard-rescue-tier depth,width,restarts
  --pair-region-table-depth N
  --pair-region-forward-depth N
  --pair-region-time-limit-ms N
  --pair-region-suffix-time-limit-ms N
  --pair-region-tier depth,width,restarts
  --pair-region-prefixes N
  --projection-kinds all|face-home,face-pair-home,tape-pair-segments
  --phase-kinds all|single-tape,cross-axis-tape-pair,three-axis-tape-triplet,pair-tape-segments,axis-pair-tape-segments,all-pair-tape-segments,protected-corner-arms,protected-corner-block,opposite-pair,opposite-android-pair,opposite-android-near-pair,opposite-android-region-pair,all-opposite-android-near-pairs,binary-color-split,opposite-layer-band,opposite-layer-class-band,adjacent-pair,corner-triplet,one-face
  --phase-prefixes N
  --phase-prefix-offset N
  --phase-tier depth,width,restarts
  --phase-time-limit-ms N
  --phase-suffix-time-limit-ms N
  --phase-profile-portfolio
  --phase-near-misses N
  --phase-probe-prefixes N
  --phase-prefix-max-over-min N
  --phase-prefix-rank length|global|combined|lookahead|pattern-distance|pattern-lookahead|suffix-probe
  --phase-rank-lookahead-depth N
  --phase-rank-lookahead-width N
  --phase-prefix-suffix-probe-time-limit-ms N
  --phase-prefix-suffix-probe-candidates N
  --phase-spec-filter TEXT[,TEXT]
  --phase-direct-threshold N
  --phase-stop-after-gain N
  --phase-skip-direct
  --phase-corner-shielded
  --phase-corner-shielded-body-depth 1|2
  --phase-corner-seed-branches N
  --phase-corner-arm-branches N
  --phase-corner-pool-specs
  --phase-suffix-hard-rescue
  --macro-shifts 3,6
  --macro-table-depth N
  --macro-pattern-depth N
  --macro-require-suffix
  --commutator-max-len N
  --commutator-top N
  --commutator-greedy-steps N
  --commutator-dynamic-target
  --commutator-plateau-lookahead N
  --commutator-suffix-rescue
  --commutator-suffix-time-limit-ms N
  --commutator-endgame-depth N
  --commutator-endgame-width N
  --commutator-endgame-time-limit-ms N
  --axis-ring-rescue
  --axis-ring-rescue-threshold N
  --axis-ring-rescue-time-limit-ms N
  --axis-ring-rescue-table-depth N
  --axis-ring-rescue-pattern-depth N
  --axis-ring-rescue-expand-depth N
  --axis-ring-rescue-tier D,W,R
  --axis-ring-rescue-position after-cascade|before-corner
  --axis-ring-rescue-corner-skip-threshold N
  --pdb-seed-source axis-ring|ariadne-midpoints|random-walk
  --pdb-seed-count N
  --pdb-seed-step-start N
  --pdb-seed-step-end N
  --pdb-random-walk-min N
  --pdb-random-walk-max N
  --feature-repeats N
  --region-pair-weight N
  --axis-ring-profile-weight N
  --axis-ring-order-weight N
  --target-expand-depth N
  --rescue-threshold N
  --rescue-time-limit-ms N
  --max-nodes N
  --time-limit-ms N
  --out reports
  --include-scripts
  --allow-same-tape
  --landmark-rescue
  --hard-rescue
  --pair-region-rescue
  --pair-region-preserve-suffix
  --no-operation-portfolio
  --no-pattern-db
  --no-f-pattern-db-portfolio
  --no-landmark-rescue
  --no-hard-rescue
  --no-pair-region-rescue
  --no-pair-region-preserve-suffix
  --no-phase-profile-portfolio
  --no-rescue
  --no-optimize
"#
    );
}

fn parse_bench_options(args: Vec<String>) -> io::Result<BenchOptions> {
    let mut layouts = LayoutId::all();
    let mut difficulties = Difficulty::all();
    let mut scramble_lengths = vec![5, 10, 20];
    let mut iterations = 3usize;
    let mut iteration_list = None;
    let mut iteration_start = 0usize;
    let mut seed = 0x5eed_2026_u64;
    let mut scramble_profile = ScrambleProfile::Lab;
    let mut target_mode = TargetMode::Android;
    let mut table_depth = None;
    let mut forward_depth = None;
    let mut path_penalty = 30_i32;
    let mut beam_rank = BeamRankMode::Score;
    let mut corridor_diversity_enabled = false;
    let mut corridor_prefix_len = DEFAULT_CORRIDOR_PREFIX_LEN;
    let mut corridor_quota_percent = DEFAULT_CORRIDOR_QUOTA_PERCENT;
    let mut local_window = 8usize;
    let mut local_depth = 6usize;
    let mut hit_patience = 0usize;
    let mut hit_restart_patience = 0usize;
    let mut retrograde_suffix_beam_first_hit = false;
    let mut portfolio_first_result = false;
    let mut operation_profile = OperationProfile::Auto;
    let mut operation_portfolio_enabled = true;
    let mut operation_portfolio_time_limit_ms = DEFAULT_OPERATION_PORTFOLIO_TIME_LIMIT_MS;
    let mut operation_portfolio_threshold = DEFAULT_OPERATION_PORTFOLIO_THRESHOLD;
    let mut pattern_db_enabled = true;
    let mut f_pattern_db_portfolio_enabled = false;
    let mut pattern_db_projection = ProjectionKind::FaceHistogram;
    let mut pattern_db_depth = DEFAULT_PATTERN_DB_DEPTH;
    let mut pattern_db_weight = DEFAULT_PATTERN_DB_WEIGHT;
    let mut pattern_db_threshold = DEFAULT_PATTERN_DB_THRESHOLD;
    let mut landmark_rescue_enabled = false;
    let mut landmark_depth = DEFAULT_LANDMARK_DEPTH;
    let mut landmark_width = DEFAULT_LANDMARK_WIDTH;
    let mut landmark_candidates = DEFAULT_LANDMARK_CANDIDATES;
    let mut landmark_time_limit_ms = DEFAULT_LANDMARK_TIME_LIMIT_MS;
    let mut landmark_suffix_time_limit_ms = DEFAULT_LANDMARK_SUFFIX_TIME_LIMIT_MS;
    let mut hard_rescue_enabled = false;
    let mut hard_rescue_time_limit_ms = DEFAULT_HARD_RESCUE_TIME_LIMIT_MS;
    let mut hard_rescue_tier = Tier {
        max_depth: DEFAULT_HARD_RESCUE_DEPTH,
        width: DEFAULT_HARD_RESCUE_WIDTH,
        restarts: DEFAULT_HARD_RESCUE_RESTARTS,
    };
    let mut pair_region_rescue_enabled = false;
    let mut pair_region_table_depth = DEFAULT_PAIR_REGION_TABLE_DEPTH;
    let mut pair_region_forward_depth = DEFAULT_PAIR_REGION_FORWARD_DEPTH;
    let mut pair_region_time_limit_ms = DEFAULT_PAIR_REGION_TIME_LIMIT_MS;
    let mut pair_region_suffix_time_limit_ms = DEFAULT_PAIR_REGION_SUFFIX_TIME_LIMIT_MS;
    let mut pair_region_tier = Tier {
        max_depth: DEFAULT_PAIR_REGION_DEPTH,
        width: DEFAULT_PAIR_REGION_WIDTH,
        restarts: DEFAULT_PAIR_REGION_RESTARTS,
    };
    let mut pair_region_prefixes = DEFAULT_PAIR_REGION_PREFIXES;
    let mut pair_region_preserve_suffix = false;
    let mut region_pair_weight = 0_i32;
    let mut axis_ring_profile_weight = 0_i32;
    let mut axis_ring_order_weight = 0_i32;
    let mut target_expand_depth = 0usize;
    let mut max_nodes = 100_000_000_u64;
    let mut time_limit_ms = 30_000_u64;
    let mut rescue_enabled = true;
    let mut rescue_threshold = DEFAULT_RESCUE_THRESHOLD;
    let mut rescue_time_limit_ms = DEFAULT_RESCUE_TIME_LIMIT_MS;
    let mut out_dir = PathBuf::from("reports");
    let mut avoid_same_tape = true;
    let mut include_scripts = false;
    let mut projection_kinds = ProjectionKind::all();
    let mut phase_kinds = PhaseKind::defaults();
    let mut phase_prefixes = DEFAULT_PHASE_PREFIXES;
    let mut phase_prefix_offset = 0usize;
    let mut phase_probe_prefixes = DEFAULT_PHASE_PREFIXES;
    let mut phase_prefix_max_over_min = None;
    let mut phase_prefix_rank = PhasePrefixRank::Length;
    let mut phase_rank_lookahead_depth = 4usize;
    let mut phase_rank_lookahead_width = 256usize;
    let mut phase_prefix_suffix_probe_time_limit_ms =
        DEFAULT_PHASE_PREFIX_SUFFIX_PROBE_TIME_LIMIT_MS;
    let mut phase_prefix_suffix_probe_candidates = DEFAULT_PHASE_PREFIX_SUFFIX_PROBE_CANDIDATES;
    let mut phase_tier = Tier {
        max_depth: DEFAULT_PHASE_DEPTH,
        width: DEFAULT_PHASE_WIDTH,
        restarts: DEFAULT_PHASE_RESTARTS,
    };
    let mut phase_time_limit_ms = DEFAULT_PHASE_TIME_LIMIT_MS;
    let mut phase_suffix_time_limit_ms = DEFAULT_PHASE_SUFFIX_TIME_LIMIT_MS;
    let mut phase_profile_portfolio = false;
    let mut phase_near_misses = DEFAULT_PHASE_NEAR_MISSES;
    let mut phase_spec_filters = Vec::new();
    let mut phase_direct_threshold = 0usize;
    let mut phase_stop_after_gain = None;
    let mut phase_skip_direct = false;
    let mut phase_corner_shielded = false;
    let mut phase_corner_shielded_body_depth = 1usize;
    let mut phase_corner_seed_branches = 1usize;
    let mut phase_corner_arm_branches = 1usize;
    let mut phase_corner_pool_specs = false;
    let mut phase_suffix_hard_rescue = false;
    let mut macro_shifts = vec![3_usize, 6];
    let mut macro_table_depth = None;
    let mut macro_pattern_depth = 0usize;
    let mut macro_require_suffix = false;
    let mut commutator_max_len = DEFAULT_COMMUTATOR_MAX_LEN;
    let mut commutator_top = DEFAULT_COMMUTATOR_TOP;
    let mut commutator_greedy_steps = DEFAULT_COMMUTATOR_GREEDY_STEPS;
    let mut commutator_dynamic_target = false;
    let mut commutator_plateau_lookahead = DEFAULT_COMMUTATOR_PLATEAU_LOOKAHEAD;
    let mut commutator_suffix_rescue = false;
    let mut commutator_suffix_time_limit_ms = DEFAULT_COMMUTATOR_SUFFIX_TIME_LIMIT_MS;
    let mut commutator_endgame_depth = DEFAULT_COMMUTATOR_ENDGAME_DEPTH;
    let mut commutator_endgame_width = DEFAULT_COMMUTATOR_ENDGAME_WIDTH;
    let mut commutator_endgame_time_limit_ms = DEFAULT_COMMUTATOR_ENDGAME_TIME_LIMIT_MS;
    let mut axis_ring_rescue_enabled = false;
    let mut axis_ring_rescue_threshold = DEFAULT_AXIS_RING_RESCUE_THRESHOLD;
    let mut axis_ring_rescue_time_limit_ms = DEFAULT_AXIS_RING_RESCUE_TIME_LIMIT_MS;
    let mut axis_ring_rescue_table_depth = DEFAULT_AXIS_RING_RESCUE_TABLE_DEPTH;
    let mut axis_ring_rescue_pattern_depth = DEFAULT_AXIS_RING_RESCUE_PATTERN_DEPTH;
    let mut axis_ring_rescue_expand_depth = DEFAULT_AXIS_RING_RESCUE_EXPAND_DEPTH;
    let mut axis_ring_rescue_tier = Tier {
        max_depth: DEFAULT_AXIS_RING_RESCUE_DEPTH,
        width: DEFAULT_AXIS_RING_RESCUE_WIDTH,
        restarts: DEFAULT_AXIS_RING_RESCUE_RESTARTS,
    };
    let mut axis_ring_rescue_position = AxisRingRescuePosition::AfterCascade;
    let mut axis_ring_rescue_corner_skip_threshold = DEFAULT_AXIS_RING_RESCUE_CORNER_SKIP_THRESHOLD;
    let mut pdb_seed_source = PdbSeedSource::AxisRing;
    let mut pdb_seed_count = DEFAULT_PDB_SEED_COUNT;
    let mut pdb_seed_step_start = DEFAULT_PDB_SEED_STEP_START;
    let mut pdb_seed_step_end = DEFAULT_PDB_SEED_STEP_END;
    let mut pdb_random_walk_min = DEFAULT_PDB_RANDOM_WALK_MIN;
    let mut pdb_random_walk_max = DEFAULT_PDB_RANDOM_WALK_MAX;
    let mut feature_repeats = DEFAULT_FEATURE_COST_REPEATS;
    let mut optimize = true;
    let mut tiers = default_tiers();

    let mut i = 0usize;
    while i < args.len() {
        let key = &args[i];
        let value = |i: &mut usize| -> io::Result<String> {
            *i += 1;
            args.get(*i).cloned().ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("{key} requires a value"),
                )
            })
        };

        match key.as_str() {
            "--layouts" => {
                let raw = value(&mut i)?;
                layouts = parse_list(&raw, LayoutId::parse, "layout")?;
            }
            "--difficulties" => {
                let raw = value(&mut i)?;
                difficulties = parse_list(&raw, Difficulty::parse, "difficulty")?;
            }
            "--scrambles" | "--scramble-lengths" => {
                let raw = value(&mut i)?;
                scramble_lengths = parse_usize_list(&raw, "scramble length")?;
            }
            "--iterations" => {
                iterations = parse_usize(&value(&mut i)?, "iterations")?;
            }
            "--iteration-list" | "--iterations-list" => {
                let raw = value(&mut i)?;
                let display_iterations = parse_usize_list(&raw, "iteration list")?;
                let mut zero_based = Vec::with_capacity(display_iterations.len());
                for display_iteration in display_iterations {
                    if display_iteration == 0 {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "iteration list values must be 1 or greater",
                        ));
                    }
                    zero_based.push(display_iteration - 1);
                }
                iteration_list = Some(zero_based);
            }
            "--iteration-start" => {
                let display_start = parse_usize(&value(&mut i)?, "iteration start")?;
                if display_start == 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "iteration start must be 1 or greater",
                    ));
                }
                iteration_start = display_start - 1;
            }
            "--seed" => {
                seed = parse_u64(&value(&mut i)?, "seed")?;
            }
            "--scramble-profile" => {
                let raw = value(&mut i)?;
                scramble_profile = ScrambleProfile::parse(&raw).ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("bad scramble profile: {raw}"),
                    )
                })?;
            }
            "--target" => {
                let raw = value(&mut i)?;
                target_mode = TargetMode::parse(&raw).ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, format!("bad target: {raw}"))
                })?;
            }
            "--table-depth" => {
                table_depth = Some(parse_usize(&value(&mut i)?, "table depth")?);
            }
            "--forward-depth" => {
                forward_depth = Some(parse_usize(&value(&mut i)?, "forward depth")?);
            }
            "--path-penalty" => {
                path_penalty = parse_i32(&value(&mut i)?, "path penalty")?;
            }
            "--beam-rank" => {
                let raw = value(&mut i)?;
                beam_rank = BeamRankMode::parse(&raw).ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, format!("bad beam rank: {raw}"))
                })?;
            }
            "--corridor-diversity" => {
                corridor_diversity_enabled = true;
            }
            "--corridor-prefix-len" => {
                corridor_prefix_len = parse_usize(&value(&mut i)?, "corridor prefix len")?;
                if corridor_prefix_len == 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "corridor prefix len must be greater than zero",
                    ));
                }
            }
            "--corridor-quota-percent" => {
                corridor_quota_percent = parse_usize(&value(&mut i)?, "corridor quota percent")?;
            }
            "--local-window" => {
                local_window = parse_usize(&value(&mut i)?, "local window")?;
            }
            "--local-depth" => {
                local_depth = parse_usize(&value(&mut i)?, "local depth")?;
            }
            "--hit-patience" => {
                hit_patience = parse_usize(&value(&mut i)?, "hit patience")?;
            }
            "--hit-restarts" => {
                hit_restart_patience = parse_usize(&value(&mut i)?, "hit restarts")?;
            }
            "--retrograde-suffix-beam-first-hit" | "--table-hit-beam-first-hit" => {
                retrograde_suffix_beam_first_hit = true;
            }
            "--no-retrograde-suffix-beam-first-hit" | "--no-table-hit-beam-first-hit" => {
                retrograde_suffix_beam_first_hit = false;
            }
            "--portfolio-first-result" | "--retrograde-suffix-portfolio-first-result" => {
                portfolio_first_result = true;
            }
            "--no-portfolio-first-result" | "--no-retrograde-suffix-portfolio-first-result" => {
                portfolio_first_result = false;
            }
            "--retrograde-suffix-early-stop" | "--table-hit-early-stop" => {
                retrograde_suffix_beam_first_hit = true;
                portfolio_first_result = true;
            }
            "--no-retrograde-suffix-early-stop" | "--no-table-hit-early-stop" => {
                retrograde_suffix_beam_first_hit = false;
                portfolio_first_result = false;
            }
            "--operation-profile" | "--ops" => {
                let raw = value(&mut i)?;
                operation_profile = OperationProfile::parse(&raw).ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("bad operation profile: {raw}"),
                    )
                })?;
            }
            "--region-pair-weight" => {
                region_pair_weight = parse_i32(&value(&mut i)?, "region pair weight")?;
            }
            "--axis-ring-profile-weight" => {
                axis_ring_profile_weight = parse_i32(&value(&mut i)?, "axis ring profile weight")?;
            }
            "--axis-ring-order-weight" => {
                axis_ring_order_weight = parse_i32(&value(&mut i)?, "axis ring order weight")?;
            }
            "--target-expand-depth" => {
                target_expand_depth = parse_usize(&value(&mut i)?, "target expand depth")?;
            }
            "--operation-portfolio-time-limit-ms" => {
                operation_portfolio_time_limit_ms =
                    parse_u64(&value(&mut i)?, "operation portfolio time limit")?;
            }
            "--operation-portfolio-threshold" => {
                operation_portfolio_threshold =
                    parse_usize(&value(&mut i)?, "operation portfolio threshold")?;
            }
            "--pattern-db-depth" => {
                pattern_db_depth = parse_usize(&value(&mut i)?, "pattern db depth")?;
            }
            "--pattern-db-weight" => {
                pattern_db_weight = parse_i32(&value(&mut i)?, "pattern db weight")?;
            }
            "--pattern-db-threshold" => {
                pattern_db_threshold = parse_usize(&value(&mut i)?, "pattern db threshold")?;
            }
            "--pattern-db-projection" => {
                let raw = value(&mut i)?;
                pattern_db_projection = ProjectionKind::parse(&raw).ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("bad pattern db projection: {raw}"),
                    )
                })?;
            }
            "--f-pattern-db-portfolio" => {
                f_pattern_db_portfolio_enabled = true;
            }
            "--landmark-depth" => {
                landmark_depth = parse_usize(&value(&mut i)?, "landmark depth")?;
            }
            "--landmark-width" => {
                landmark_width = parse_usize(&value(&mut i)?, "landmark width")?;
            }
            "--landmark-candidates" => {
                landmark_candidates = parse_usize(&value(&mut i)?, "landmark candidates")?;
            }
            "--landmark-time-limit-ms" => {
                landmark_time_limit_ms = parse_u64(&value(&mut i)?, "landmark time limit")?;
            }
            "--landmark-suffix-time-limit-ms" => {
                landmark_suffix_time_limit_ms =
                    parse_u64(&value(&mut i)?, "landmark suffix time limit")?;
            }
            "--hard-rescue-time-limit-ms" => {
                hard_rescue_time_limit_ms = parse_u64(&value(&mut i)?, "hard rescue time limit")?;
            }
            "--hard-rescue-tier" => {
                let raw = value(&mut i)?;
                let mut tiers = parse_tiers(&raw)?;
                if tiers.len() != 1 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "hard rescue tier must be one depth,width,restarts triple",
                    ));
                }
                hard_rescue_tier = tiers.remove(0);
            }
            "--pair-region-table-depth" => {
                pair_region_table_depth = parse_usize(&value(&mut i)?, "pair region table depth")?;
            }
            "--pair-region-forward-depth" => {
                pair_region_forward_depth =
                    parse_usize(&value(&mut i)?, "pair region forward depth")?;
            }
            "--pair-region-time-limit-ms" => {
                pair_region_time_limit_ms = parse_u64(&value(&mut i)?, "pair region time limit")?;
            }
            "--pair-region-suffix-time-limit-ms" => {
                pair_region_suffix_time_limit_ms =
                    parse_u64(&value(&mut i)?, "pair region suffix time limit")?;
            }
            "--pair-region-tier" => {
                let raw = value(&mut i)?;
                let mut tiers = parse_tiers(&raw)?;
                if tiers.len() != 1 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "pair region tier must be one depth,width,restarts triple",
                    ));
                }
                pair_region_tier = tiers.remove(0);
            }
            "--pair-region-prefixes" => {
                pair_region_prefixes = parse_usize(&value(&mut i)?, "pair region prefixes")?;
            }
            "--phase-kinds" => {
                phase_kinds = parse_phase_kinds(&value(&mut i)?)?;
            }
            "--phase-prefixes" => {
                phase_prefixes = parse_usize(&value(&mut i)?, "phase prefixes")?;
            }
            "--phase-prefix-offset" => {
                phase_prefix_offset = parse_usize(&value(&mut i)?, "phase prefix offset")?;
            }
            "--phase-probe-prefixes" => {
                phase_probe_prefixes = parse_usize(&value(&mut i)?, "phase probe prefixes")?;
            }
            "--phase-prefix-max-over-min" => {
                phase_prefix_max_over_min =
                    Some(parse_usize(&value(&mut i)?, "phase prefix max over min")?);
            }
            "--phase-prefix-rank" => {
                let raw = value(&mut i)?;
                phase_prefix_rank = PhasePrefixRank::parse(&raw).ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("bad phase prefix rank: {raw}"),
                    )
                })?;
            }
            "--phase-rank-lookahead-depth" => {
                phase_rank_lookahead_depth =
                    parse_usize(&value(&mut i)?, "phase rank lookahead depth")?;
            }
            "--phase-rank-lookahead-width" => {
                phase_rank_lookahead_width =
                    parse_usize(&value(&mut i)?, "phase rank lookahead width")?;
                if phase_rank_lookahead_width == 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "phase rank lookahead width must be greater than zero",
                    ));
                }
            }
            "--phase-prefix-suffix-probe-time-limit-ms" => {
                phase_prefix_suffix_probe_time_limit_ms =
                    parse_u64(&value(&mut i)?, "phase prefix suffix probe time limit")?;
            }
            "--phase-prefix-suffix-probe-candidates" => {
                phase_prefix_suffix_probe_candidates =
                    parse_usize(&value(&mut i)?, "phase prefix suffix probe candidates")?;
            }
            "--phase-tier" => {
                let raw = value(&mut i)?;
                let mut tiers = parse_tiers(&raw)?;
                if tiers.len() != 1 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "phase tier must be one depth,width,restarts triple",
                    ));
                }
                phase_tier = tiers.remove(0);
            }
            "--phase-time-limit-ms" => {
                phase_time_limit_ms = parse_u64(&value(&mut i)?, "phase time limit")?;
            }
            "--phase-suffix-time-limit-ms" => {
                phase_suffix_time_limit_ms = parse_u64(&value(&mut i)?, "phase suffix time limit")?;
            }
            "--phase-profile-portfolio" => {
                phase_profile_portfolio = true;
            }
            "--phase-near-misses" => {
                phase_near_misses = parse_usize(&value(&mut i)?, "phase near misses")?;
            }
            "--phase-spec-filter" | "--phase-filter" => {
                phase_spec_filters = parse_string_list(&value(&mut i)?);
            }
            "--phase-direct-threshold" => {
                phase_direct_threshold = parse_usize(&value(&mut i)?, "phase direct threshold")?;
            }
            "--phase-stop-after-gain" => {
                phase_stop_after_gain =
                    Some(parse_usize(&value(&mut i)?, "phase stop after gain")?);
            }
            "--phase-skip-direct" | "--no-phase-direct" => phase_skip_direct = true,
            "--phase-corner-shielded" => phase_corner_shielded = true,
            "--phase-corner-shielded-body-depth" => {
                phase_corner_shielded_body_depth =
                    parse_usize(&value(&mut i)?, "phase corner shielded body depth")?;
                if !(1..=2).contains(&phase_corner_shielded_body_depth) {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "phase corner shielded body depth must be 1 or 2",
                    ));
                }
            }
            "--phase-corner-seed-branches" => {
                phase_corner_seed_branches =
                    parse_usize(&value(&mut i)?, "phase corner seed branches")?;
                if phase_corner_seed_branches == 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "phase corner seed branches must be greater than zero",
                    ));
                }
            }
            "--phase-corner-arm-branches" => {
                phase_corner_arm_branches =
                    parse_usize(&value(&mut i)?, "phase corner arm branches")?;
                if phase_corner_arm_branches == 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "phase corner arm branches must be greater than zero",
                    ));
                }
            }
            "--phase-corner-pool-specs" => phase_corner_pool_specs = true,
            "--phase-suffix-hard-rescue" => phase_suffix_hard_rescue = true,
            "--macro-shifts" => {
                macro_shifts = parse_usize_list(&value(&mut i)?, "macro shifts")?;
                if macro_shifts.iter().any(|shift| *shift == 0) {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "macro shifts must be greater than zero",
                    ));
                }
            }
            "--macro-table-depth" => {
                macro_table_depth = Some(parse_usize(&value(&mut i)?, "macro table depth")?);
            }
            "--macro-pattern-depth" => {
                macro_pattern_depth = parse_usize(&value(&mut i)?, "macro pattern depth")?;
            }
            "--macro-require-suffix" => {
                macro_require_suffix = true;
            }
            "--commutator-max-len" => {
                commutator_max_len = parse_usize(&value(&mut i)?, "commutator max len")?;
                if commutator_max_len == 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "commutator max len must be greater than zero",
                    ));
                }
            }
            "--commutator-top" => {
                commutator_top = parse_usize(&value(&mut i)?, "commutator top")?;
                if commutator_top == 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "commutator top must be greater than zero",
                    ));
                }
            }
            "--commutator-greedy-steps" => {
                commutator_greedy_steps = parse_usize(&value(&mut i)?, "commutator greedy steps")?;
            }
            "--commutator-dynamic-target" => commutator_dynamic_target = true,
            "--commutator-plateau-lookahead" => {
                commutator_plateau_lookahead =
                    parse_usize(&value(&mut i)?, "commutator plateau lookahead")?;
            }
            "--commutator-suffix-rescue" => commutator_suffix_rescue = true,
            "--commutator-suffix-time-limit-ms" => {
                commutator_suffix_time_limit_ms =
                    parse_u64(&value(&mut i)?, "commutator suffix time limit")?;
            }
            "--commutator-endgame-depth" => {
                commutator_endgame_depth =
                    parse_usize(&value(&mut i)?, "commutator endgame depth")?;
            }
            "--commutator-endgame-width" => {
                commutator_endgame_width =
                    parse_usize(&value(&mut i)?, "commutator endgame width")?;
                if commutator_endgame_width == 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "commutator endgame width must be greater than zero",
                    ));
                }
            }
            "--commutator-endgame-time-limit-ms" => {
                commutator_endgame_time_limit_ms =
                    parse_u64(&value(&mut i)?, "commutator endgame time limit")?;
            }
            "--axis-ring-rescue" => {
                axis_ring_rescue_enabled = true;
            }
            "--axis-ring-rescue-threshold" => {
                axis_ring_rescue_threshold =
                    parse_usize(&value(&mut i)?, "axis ring rescue threshold")?;
            }
            "--axis-ring-rescue-time-limit-ms" => {
                axis_ring_rescue_time_limit_ms =
                    parse_u64(&value(&mut i)?, "axis ring rescue time limit")?;
            }
            "--axis-ring-rescue-table-depth" => {
                axis_ring_rescue_table_depth =
                    parse_usize(&value(&mut i)?, "axis ring rescue table depth")?;
            }
            "--axis-ring-rescue-pattern-depth" => {
                axis_ring_rescue_pattern_depth =
                    parse_usize(&value(&mut i)?, "axis ring rescue pattern depth")?;
            }
            "--axis-ring-rescue-expand-depth" => {
                axis_ring_rescue_expand_depth =
                    parse_usize(&value(&mut i)?, "axis ring rescue expand depth")?;
            }
            "--axis-ring-rescue-tier" => {
                let raw = value(&mut i)?;
                let mut tiers = parse_tiers(&raw)?;
                if tiers.len() != 1 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "axis ring rescue tier must be one depth,width,restarts triple",
                    ));
                }
                axis_ring_rescue_tier = tiers.remove(0);
            }
            "--axis-ring-rescue-position" => {
                let raw = value(&mut i)?;
                axis_ring_rescue_position =
                    AxisRingRescuePosition::parse(&raw).ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidInput,
                            format!("bad axis ring rescue position: {raw}"),
                        )
                    })?;
            }
            "--axis-ring-rescue-corner-skip-threshold" => {
                axis_ring_rescue_corner_skip_threshold =
                    parse_usize(&value(&mut i)?, "axis ring rescue corner skip threshold")?;
            }
            "--pdb-seed-source" => {
                let raw = value(&mut i)?;
                pdb_seed_source = PdbSeedSource::parse(&raw).ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("bad PDB seed source: {raw}"),
                    )
                })?;
            }
            "--pdb-seed-count" => {
                pdb_seed_count = parse_usize(&value(&mut i)?, "PDB seed count")?;
            }
            "--pdb-seed-step-start" => {
                pdb_seed_step_start = parse_usize(&value(&mut i)?, "PDB seed step start")?;
            }
            "--pdb-seed-step-end" => {
                pdb_seed_step_end = parse_usize(&value(&mut i)?, "PDB seed step end")?;
            }
            "--pdb-random-walk-min" => {
                pdb_random_walk_min = parse_usize(&value(&mut i)?, "PDB random walk min")?;
            }
            "--pdb-random-walk-max" => {
                pdb_random_walk_max = parse_usize(&value(&mut i)?, "PDB random walk max")?;
            }
            "--feature-repeats" => {
                feature_repeats = parse_usize(&value(&mut i)?, "feature repeats")?;
                if feature_repeats == 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "feature repeats must be greater than zero",
                    ));
                }
            }
            "--max-nodes" => {
                max_nodes = parse_u64(&value(&mut i)?, "max nodes")?;
            }
            "--time-limit-ms" => {
                time_limit_ms = parse_u64(&value(&mut i)?, "time limit")?;
            }
            "--rescue-threshold" => {
                rescue_threshold = parse_usize(&value(&mut i)?, "rescue threshold")?;
            }
            "--rescue-time-limit-ms" => {
                rescue_time_limit_ms = parse_u64(&value(&mut i)?, "rescue time limit")?;
            }
            "--out" => {
                out_dir = PathBuf::from(value(&mut i)?);
            }
            "--tier" => {
                let raw = value(&mut i)?;
                tiers = parse_tiers(&raw)?;
            }
            "--include-scripts" => include_scripts = true,
            "--projection-kinds" => {
                projection_kinds = parse_projection_kinds(&value(&mut i)?)?;
            }
            "--allow-same-tape" => avoid_same_tape = false,
            "--landmark-rescue" => landmark_rescue_enabled = true,
            "--hard-rescue" => hard_rescue_enabled = true,
            "--pair-region-rescue" => pair_region_rescue_enabled = true,
            "--pair-region-preserve-suffix" => pair_region_preserve_suffix = true,
            "--no-operation-portfolio" => operation_portfolio_enabled = false,
            "--no-pattern-db" => pattern_db_enabled = false,
            "--no-f-pattern-db-portfolio" => f_pattern_db_portfolio_enabled = false,
            "--no-landmark-rescue" => landmark_rescue_enabled = false,
            "--no-hard-rescue" => hard_rescue_enabled = false,
            "--no-pair-region-rescue" => pair_region_rescue_enabled = false,
            "--no-pair-region-preserve-suffix" => pair_region_preserve_suffix = false,
            "--no-axis-ring-rescue" => axis_ring_rescue_enabled = false,
            "--no-phase-profile-portfolio" => phase_profile_portfolio = false,
            "--no-rescue" => rescue_enabled = false,
            "--no-optimize" => optimize = false,
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            other => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("unknown option: {other}"),
                ));
            }
        }
        i += 1;
    }

    Ok(BenchOptions {
        layouts,
        difficulties,
        scramble_lengths,
        iterations,
        iteration_list,
        iteration_start,
        seed,
        scramble_profile,
        out_dir,
        avoid_same_tape,
        include_scripts,
        projection_kinds,
        phase_kinds,
        phase_prefixes,
        phase_prefix_offset,
        phase_probe_prefixes,
        phase_prefix_max_over_min,
        phase_prefix_rank,
        phase_rank_lookahead_depth,
        phase_rank_lookahead_width,
        phase_prefix_suffix_probe_time_limit_ms,
        phase_prefix_suffix_probe_candidates,
        phase_tier,
        phase_time_limit_ms,
        phase_suffix_time_limit_ms,
        phase_profile_portfolio,
        phase_near_misses,
        phase_spec_filters,
        phase_direct_threshold,
        phase_stop_after_gain,
        phase_skip_direct,
        phase_corner_shielded,
        phase_corner_shielded_body_depth,
        phase_corner_seed_branches,
        phase_corner_arm_branches,
        phase_corner_pool_specs,
        phase_suffix_hard_rescue,
        macro_shifts,
        macro_table_depth,
        macro_pattern_depth,
        macro_require_suffix,
        commutator_max_len,
        commutator_top,
        commutator_greedy_steps,
        commutator_dynamic_target,
        commutator_plateau_lookahead,
        commutator_suffix_rescue,
        commutator_suffix_time_limit_ms,
        commutator_endgame_depth,
        commutator_endgame_width,
        commutator_endgame_time_limit_ms,
        axis_ring_rescue_enabled,
        axis_ring_rescue_threshold,
        axis_ring_rescue_time_limit_ms,
        axis_ring_rescue_table_depth,
        axis_ring_rescue_pattern_depth,
        axis_ring_rescue_expand_depth,
        axis_ring_rescue_tier,
        axis_ring_rescue_position,
        axis_ring_rescue_corner_skip_threshold,
        pdb_seed_source,
        pdb_seed_count,
        pdb_seed_step_start,
        pdb_seed_step_end,
        pdb_random_walk_min,
        pdb_random_walk_max,
        feature_repeats,
        e_classic_cascade: false,
        solver: SolverConfig {
            target_mode,
            table_depth,
            forward_depth,
            tiers,
            path_penalty,
            beam_rank,
            corridor_diversity_enabled,
            corridor_prefix_len,
            corridor_quota_percent,
            local_window,
            local_depth,
            hit_patience,
            hit_restart_patience,
            retrograde_suffix_beam_first_hit,
            portfolio_first_result,
            operation_profile,
            operation_portfolio_enabled,
            operation_portfolio_time_limit_ms,
            operation_portfolio_threshold,
            pattern_db_enabled,
            f_pattern_db_portfolio_enabled,
            pattern_db_projection,
            pattern_db_depth,
            pattern_db_weight,
            pattern_db_threshold,
            landmark_rescue_enabled,
            landmark_depth,
            landmark_width,
            landmark_candidates,
            landmark_time_limit_ms,
            landmark_suffix_time_limit_ms,
            hard_rescue_enabled,
            hard_rescue_time_limit_ms,
            hard_rescue_tier,
            pair_region_rescue_enabled,
            pair_region_table_depth,
            pair_region_forward_depth,
            pair_region_time_limit_ms,
            pair_region_suffix_time_limit_ms,
            pair_region_tier,
            pair_region_prefixes,
            pair_region_preserve_suffix,
            region_pair_weight,
            axis_ring_profile_weight,
            axis_ring_order_weight,
            target_expand_depth,
            max_nodes,
            time_limit_ms,
            rescue_enabled,
            rescue_threshold,
            rescue_time_limit_ms,
            optimize,
        },
    })
}

fn parse_e_classic_cascade_options(args: Vec<String>) -> io::Result<BenchOptions> {
    let skip_direct_requested = args.iter().any(|arg| arg == "--phase-skip-direct");
    let mut profile_args = vec![
        "--scrambles".to_string(),
        "80".to_string(),
        "--target".to_string(),
        "android-portfolio".to_string(),
        "--max-nodes".to_string(),
        "700000000".to_string(),
        "--time-limit-ms".to_string(),
        "60000".to_string(),
    ];
    profile_args.extend(args);

    let mut options = parse_bench_options(profile_args)?;
    options.layouts = vec![LayoutId::E];
    options.difficulties = vec![Difficulty::Classic];
    options.phase_kinds = vec![
        PhaseKind::AllOppositeAndroidNearPairs,
        PhaseKind::ProtectedCornerArms,
    ];
    options.phase_skip_direct = skip_direct_requested;
    options.phase_direct_threshold = 0;
    options.e_classic_cascade = true;
    Ok(options)
}

fn e_classic_near_pair_options(base: &BenchOptions) -> BenchOptions {
    let mut options = base.clone();
    options.phase_kinds = vec![PhaseKind::AllOppositeAndroidNearPairs];
    options.phase_prefixes = 4;
    options.phase_prefix_offset = 0;
    options.phase_probe_prefixes = 16;
    options.phase_prefix_max_over_min = None;
    options.phase_prefix_rank = PhasePrefixRank::Global;
    options.phase_tier = Tier {
        max_depth: 160,
        width: 1500,
        restarts: 3,
    };
    options.phase_time_limit_ms = 30_000;
    options.phase_suffix_time_limit_ms = 20_000;
    options.phase_profile_portfolio = false;
    options.phase_near_misses = 2;
    options.phase_stop_after_gain = Some(10);
    options.phase_corner_pool_specs = false;
    options.phase_suffix_hard_rescue = false;
    options
}

fn e_classic_corner_options(base: &BenchOptions) -> BenchOptions {
    let mut options = base.clone();
    options.phase_kinds = vec![PhaseKind::ProtectedCornerArms];
    options.phase_prefixes = E_CLASSIC_CORNER_PRIMARY_PREFIXES + E_CLASSIC_CORNER_FALLBACK_PREFIXES;
    options.phase_prefix_offset = 0;
    options.phase_probe_prefixes = 16;
    options.phase_prefix_max_over_min = Some(1);
    options.phase_prefix_rank = PhasePrefixRank::SuffixProbe;
    options.phase_rank_lookahead_depth = 5;
    options.phase_rank_lookahead_width = 256;
    options.phase_prefix_suffix_probe_time_limit_ms = 5_000;
    options.phase_prefix_suffix_probe_candidates = 4;
    options.phase_tier = Tier {
        max_depth: 180,
        width: 1800,
        restarts: 4,
    };
    options.phase_time_limit_ms = 90_000;
    options.phase_suffix_time_limit_ms = 30_000;
    options.phase_profile_portfolio = false;
    options.phase_near_misses = 0;
    options.phase_stop_after_gain = Some(10);
    options.phase_corner_shielded = true;
    options.phase_corner_shielded_body_depth = 1;
    options.phase_corner_seed_branches = 1;
    options.phase_corner_arm_branches = 4;
    options.phase_corner_pool_specs = true;
    options.phase_suffix_hard_rescue = false;
    options
}

fn should_run_e_classic_corner_rescue(
    scramble_len: usize,
    direct_found: bool,
    direct_opt_len: usize,
    current_found: bool,
    current_opt_len: usize,
) -> bool {
    if scramble_len < E_CLASSIC_CORNER_RESCUE_MIN_SCRAMBLE {
        return false;
    }

    let direct_outlier = direct_found && direct_opt_len > E_CLASSIC_CORNER_QUALITY_THRESHOLD;
    let unresolved_or_current_outlier =
        !current_found || current_opt_len > E_CLASSIC_CORNER_QUALITY_THRESHOLD;
    direct_outlier || unresolved_or_current_outlier
}

#[derive(Debug, Clone)]
struct SolveStateOptions {
    layout: LayoutId,
    difficulty: Difficulty,
    colors: Vec<Color>,
    profile: String,
    solver_args: Vec<String>,
}

fn run_solve_state(args: Vec<String>) -> io::Result<()> {
    let options = parse_solve_state_options(args)?;
    let puzzle = Puzzle::new(options.layout, options.difficulty);
    validate_state_colors(&puzzle, &options.colors)?;

    let mut solver_args = solve_state_profile_args(&options.profile, options.layout, options.difficulty)?;
    solver_args.extend(options.solver_args);
    let bench_options = parse_bench_options(solver_args)?;

    let started = Instant::now();
    let result = if is_target_solved(
        &options.colors,
        &puzzle,
        acceptance_target(bench_options.solver.target_mode),
    ) {
        SolveResult {
            found: true,
            method: SolveMethod::None,
            target_used: Some(bench_options.solver.target_mode),
            operation_profile_used: None,
            reason: "already_solved".to_string(),
            raw_moves: Vec::new(),
            optimized_moves: Vec::new(),
            nodes: 0,
            elapsed_ms: 0,
            first_table_hit: None,
        }
    } else {
        let artifacts = prepare_solver(&puzzle, &bench_options.solver)?;
        solve_puzzle(&puzzle, &options.colors, &bench_options.solver, &artifacts)
    };

    let mut solved_colors = options.colors.clone();
    puzzle.apply_moves(&mut solved_colors, &result.optimized_moves);
    let uniform_solved = is_uniform_solved(&solved_colors, &puzzle.face_indexes);
    let android_solved = is_android_solved(&solved_colors, &puzzle.face_indexes, puzzle.difficulty);
    let elapsed_ms = started.elapsed().as_millis();

    println!(
        "{{\"status\":\"{}\",\"found\":{},\"layout\":\"{}\",\"difficulty\":\"{}\",\"profile\":\"{}\",\"method\":\"{}\",\"target\":\"{}\",\"operationProfile\":\"{}\",\"reason\":\"{}\",\"moves\":{},\"text\":\"{}\",\"rawMoveCount\":{},\"moveCount\":{},\"nodes\":{},\"elapsedMs\":{},\"solverElapsedMs\":{},\"uniformSolved\":{},\"androidSolved\":{}}}",
        if result.found { "solved" } else { "not_found" },
        result.found,
        puzzle.layout,
        puzzle.difficulty,
        json_escape(&options.profile),
        result.method.label(),
        result
            .target_used
            .map(|target| target.label())
            .unwrap_or("none"),
        result
            .operation_profile_used
            .map(|profile| profile.label())
            .unwrap_or("none"),
        json_escape(&result.reason),
        moves_json(&puzzle, &result.optimized_moves),
        json_escape(&puzzle.moves_text(&result.optimized_moves)),
        result.raw_moves.len(),
        result.optimized_moves.len(),
        result.nodes,
        elapsed_ms,
        result.elapsed_ms,
        uniform_solved,
        android_solved
    );

    Ok(())
}

fn parse_solve_state_options(args: Vec<String>) -> io::Result<SolveStateOptions> {
    let mut layout = LayoutId::E;
    let mut difficulty = Difficulty::Classic;
    let mut colors = None;
    let mut profile = "balanced".to_string();
    let mut solver_args = Vec::new();

    let mut i = 0usize;
    while i < args.len() {
        let key = &args[i];
        if key == "--" {
            solver_args.extend(args[i + 1..].iter().cloned());
            break;
        }
        let mut value = || -> io::Result<String> {
            i += 1;
            args.get(i).cloned().ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("{key} requires a value"),
                )
            })
        };
        match key.as_str() {
            "--layout" => {
                let raw = value()?;
                layout = LayoutId::parse(&raw).ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, format!("bad layout: {raw}"))
                })?;
            }
            "--difficulty" => {
                let raw = value()?;
                difficulty = Difficulty::parse(&raw).ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, format!("bad difficulty: {raw}"))
                })?;
            }
            "--colors" | "--state" => {
                colors = Some(parse_color_codes(&value()?)?);
            }
            "--profile" => {
                profile = value()?.to_ascii_lowercase();
            }
            "--help" | "-h" => {
                println!(
                    "Usage: ashtree_native_bench solve-state --layout E --difficulty classic --colors 0,1,2,... [--profile fast|balanced|quality] [-- solver options]"
                );
                std::process::exit(0);
            }
            other => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("unknown solve-state option: {other}; pass solver tuning after --"),
                ));
            }
        }
        i += 1;
    }

    Ok(SolveStateOptions {
        layout,
        difficulty,
        colors: colors.ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "--colors is required")
        })?,
        profile,
        solver_args,
    })
}

fn solve_state_profile_args(
    profile: &str,
    layout: LayoutId,
    difficulty: Difficulty,
) -> io::Result<Vec<String>> {
    let mut args = match profile {
        "fast" => vec![
            "--target",
            "android-multi",
            "--beam-rank",
            "ring-rotation",
            "--operation-profile",
            "auto",
            "--operation-portfolio-time-limit-ms",
            "6000",
            "--time-limit-ms",
            "15000",
            "--max-nodes",
            "150000000",
            "--retrograde-suffix-beam-first-hit",
        ],
        "balanced" | "default" | "interactive" => vec![
            "--target",
            "android-multi",
            "--beam-rank",
            "ring-portfolio",
            "--operation-profile",
            "auto",
            "--operation-portfolio-time-limit-ms",
            "10000",
            "--time-limit-ms",
            "30000",
            "--max-nodes",
            "300000000",
            "--retrograde-suffix-beam-first-hit",
        ],
        "quality" => vec![
            "--target",
            "android-portfolio",
            "--beam-rank",
            "ring-portfolio",
            "--operation-profile",
            "auto",
            "--operation-portfolio-time-limit-ms",
            "15000",
            "--time-limit-ms",
            "60000",
            "--max-nodes",
            "700000000",
            "--retrograde-suffix-beam-first-hit",
            "--axis-ring-rescue",
            "--axis-ring-rescue-position",
            "before-corner",
            "--axis-ring-rescue-time-limit-ms",
            "30000",
        ],
        other => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("bad solve-state profile: {other}"),
            ));
        }
    };

    if layout == LayoutId::F && difficulty == Difficulty::Classic {
        args.extend([
            "--f-pattern-db-portfolio",
            "--pattern-db-depth",
            "6",
            "--pattern-db-projection",
            "face-histogram",
        ]);
    }

    Ok(args.into_iter().map(str::to_string).collect())
}

fn parse_color_codes(raw: &str) -> io::Result<Vec<Color>> {
    let raw = raw.trim().trim_start_matches('[').trim_end_matches(']');
    if raw.is_empty() {
        return Ok(Vec::new());
    }
    raw.split(|ch: char| ch == ',' || ch.is_ascii_whitespace())
        .filter(|part| !part.trim().is_empty())
        .map(|part| {
            let value = part.trim().parse::<u8>().map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("bad color code: {}", part.trim()),
                )
            })?;
            if value > YELLOW {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("color code out of range: {value}"),
                ));
            }
            Ok(value)
        })
        .collect()
}

fn validate_state_colors(puzzle: &Puzzle, colors: &[Color]) -> io::Result<()> {
    if colors.len() != puzzle.stickers.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "state has {} stickers, expected {}",
                colors.len(),
                puzzle.stickers.len()
            ),
        ));
    }
    let counts = count_colors(colors);
    if counts != puzzle.target_color_counts {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "state color counts do not match difficulty: got {:?}, expected {:?}",
                counts, puzzle.target_color_counts
            ),
        ));
    }
    Ok(())
}

fn moves_json(puzzle: &Puzzle, moves: &[MoveIndex]) -> String {
    let mut out = String::from("[");
    for (index, &move_index) in moves.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        let mv = &puzzle.moves[move_index];
        out.push_str(&format!(
            "{{\"tapeId\":\"{}\",\"direction\":{}}}",
            json_escape(&mv.tape_id),
            mv.direction
        ));
    }
    out.push(']');
    out
}

fn json_escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out
}

fn parse_list<T, F>(raw: &str, parse: F, label: &str) -> io::Result<Vec<T>>
where
    F: Fn(&str) -> Option<T>,
{
    let mut out = Vec::new();
    for part in raw.split(',') {
        let value = parse(part).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("bad {label}: {}", part.trim()),
            )
        })?;
        out.push(value);
    }
    if out.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("empty {label} list"),
        ));
    }
    Ok(out)
}

fn parse_phase_kinds(raw: &str) -> io::Result<Vec<PhaseKind>> {
    let mut out = Vec::new();
    for part in raw.split(',') {
        let kinds = PhaseKind::parse(part).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("bad phase kind: {}", part.trim()),
            )
        })?;
        for kind in kinds {
            if !out.contains(&kind) {
                out.push(kind);
            }
        }
    }
    if out.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "empty phase kind list",
        ));
    }
    Ok(out)
}

fn parse_projection_kinds(raw: &str) -> io::Result<Vec<ProjectionKind>> {
    let mut out = Vec::new();
    for part in raw.split(',') {
        let part = part.trim();
        let kinds = if part.eq_ignore_ascii_case("all") {
            ProjectionKind::all()
        } else {
            vec![ProjectionKind::parse(part).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("bad projection kind: {part}"),
                )
            })?]
        };
        for kind in kinds {
            if !out.contains(&kind) {
                out.push(kind);
            }
        }
    }
    if out.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "empty projection kind list",
        ));
    }
    Ok(out)
}

fn parse_string_list(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| part.to_ascii_lowercase())
        .collect()
}

fn parse_usize_list(raw: &str, label: &str) -> io::Result<Vec<usize>> {
    raw.split(',')
        .map(|part| parse_usize(part.trim(), label))
        .collect()
}

fn parse_usize(raw: &str, label: &str) -> io::Result<usize> {
    raw.parse::<usize>()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, format!("bad {label}: {raw}")))
}

fn parse_u64(raw: &str, label: &str) -> io::Result<u64> {
    raw.parse::<u64>()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, format!("bad {label}: {raw}")))
}

fn parse_i32(raw: &str, label: &str) -> io::Result<i32> {
    raw.parse::<i32>()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, format!("bad {label}: {raw}")))
}

fn parse_tiers(raw: &str) -> io::Result<Vec<Tier>> {
    let mut out = Vec::new();
    for part in raw.split(';') {
        let nums = parse_usize_list(part, "tier")?;
        if nums.len() != 3 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "tier must be max_depth,width,restarts",
            ));
        }
        out.push(Tier {
            max_depth: nums[0],
            width: nums[1],
            restarts: nums[2],
        });
    }
    Ok(out)
}

fn default_tiers() -> Vec<Tier> {
    vec![
        Tier {
            max_depth: 80,
            width: 700,
            restarts: 4,
        },
        Tier {
            max_depth: 105,
            width: 900,
            restarts: 3,
        },
        Tier {
            max_depth: 130,
            width: 1150,
            restarts: 2,
        },
    ]
}

fn iteration_values(options: &BenchOptions) -> Vec<usize> {
    options.iteration_list.clone().unwrap_or_else(|| {
        (options.iteration_start..options.iteration_start + options.iterations).collect()
    })
}

fn iteration_label(options: &BenchOptions) -> String {
    match &options.iteration_list {
        Some(iterations) => format!(
            "list:{}",
            iterations
                .iter()
                .map(|iteration| (iteration + 1).to_string())
                .collect::<Vec<_>>()
                .join(",")
        ),
        None => options.iterations.to_string(),
    }
}

fn run_bench(options: BenchOptions) -> io::Result<()> {
    fs::create_dir_all(&options.out_dir)?;

    let stamp = unix_stamp();
    let csv_path = options.out_dir.join(format!("bench_{stamp}.csv"));
    let md_path = options.out_dir.join(format!("bench_{stamp}.md"));
    let mut records = Vec::new();

    println!(
        "bench target={} scramble_profile={} layouts={} difficulties={} scrambles={:?} iterations={} iteration_start={} seed={}",
        options.solver.target_mode.label(),
        options.scramble_profile.label(),
        join_display(&options.layouts),
        join_display(&options.difficulties),
        options.scramble_lengths,
        iteration_label(&options),
        options.iteration_start + 1,
        options.seed
    );

    let iterations = iteration_values(&options);
    for &layout in &options.layouts {
        for &difficulty in &options.difficulties {
            let puzzle = Puzzle::new(layout, difficulty);
            let artifacts = prepare_solver(&puzzle, &options.solver)?;
            println!(
                "prepared {layout}-{difficulty}: variants={} build_nodes={} build_time={}ms [{}]",
                artifacts.variants.len(),
                artifacts.build_nodes,
                artifacts.build_ms,
                artifact_summary(&artifacts)
            );
            for &scramble_len in &options.scramble_lengths {
                for &iteration in &iterations {
                    let run_seed =
                        derive_seed(options.seed, layout, difficulty, scramble_len, iteration);
                    let scramble = generate_scramble(
                        &puzzle,
                        scramble_len,
                        run_seed,
                        options.scramble_profile,
                        options.avoid_same_tape,
                    );
                    let ariadne_solution_len = ariadne_solution_len(&puzzle, &scramble);
                    let mut scrambled_colors = puzzle.solved_colors.clone();
                    puzzle.apply_moves(&mut scrambled_colors, &scramble);

                    let result =
                        solve_puzzle(&puzzle, &scrambled_colors, &options.solver, &artifacts);

                    let mut raw_colors = scrambled_colors.clone();
                    puzzle.apply_moves(&mut raw_colors, &result.raw_moves);
                    let mut opt_colors = scrambled_colors.clone();
                    puzzle.apply_moves(&mut opt_colors, &result.optimized_moves);

                    let uniform_solved = is_uniform_solved(&opt_colors, &puzzle.face_indexes);
                    let android_solved =
                        is_android_solved(&opt_colors, &puzzle.face_indexes, difficulty);
                    let optimized_len = result.optimized_moves.len();

                    let gain = scramble_len as isize - optimized_len as isize;
                    let ratio = if scramble_len == 0 {
                        0.0
                    } else {
                        optimized_len as f64 / scramble_len as f64
                    };
                    let first_table_hit = result.first_table_hit.clone();

                    let record = RunRecord {
                        layout,
                        difficulty,
                        scramble_len,
                        iteration: iteration + 1,
                        seed: run_seed,
                        ariadne_solution_len,
                        found: result.found,
                        reason: result.reason,
                        method: result.method,
                        raw_solution_len: result.raw_moves.len(),
                        optimized_solution_len: optimized_len,
                        optimized_changed: result.raw_moves != result.optimized_moves,
                        target_used: result
                            .target_used
                            .map(|target| target.label().to_string())
                            .unwrap_or_else(|| "none".to_string()),
                        operation_profile_used: result
                            .operation_profile_used
                            .map(|profile| profile.label().to_string())
                            .unwrap_or_else(|| "none".to_string()),
                        uniform_solved,
                        android_solved,
                        nodes: result.nodes,
                        elapsed_ms: result.elapsed_ms,
                        first_table_hit: first_table_hit.is_some(),
                        first_table_hit_target: first_table_hit
                            .as_ref()
                            .map(|hit| hit.target.label().to_string())
                            .unwrap_or_else(|| "none".to_string()),
                        first_table_hit_profile: first_table_hit
                            .as_ref()
                            .map(|hit| hit.operation_profile.label().to_string())
                            .unwrap_or_else(|| "none".to_string()),
                        first_table_hit_rank: first_table_hit
                            .as_ref()
                            .map(|hit| hit.beam_rank.label().to_string())
                            .unwrap_or_else(|| "none".to_string()),
                        first_table_hit_pattern_db: first_table_hit
                            .as_ref()
                            .is_some_and(|hit| hit.pattern_db),
                        first_table_hit_depth: first_table_hit
                            .as_ref()
                            .map(|hit| hit.depth)
                            .unwrap_or_default(),
                        first_table_hit_restart: first_table_hit
                            .as_ref()
                            .map(|hit| hit.restart)
                            .unwrap_or_default(),
                        first_table_hit_nodes: first_table_hit
                            .as_ref()
                            .map(|hit| hit.nodes)
                            .unwrap_or_default(),
                        first_table_hit_elapsed_ms: first_table_hit
                            .as_ref()
                            .map(|hit| hit.elapsed_ms)
                            .unwrap_or_default(),
                        first_table_hit_prefix_len: first_table_hit
                            .as_ref()
                            .map(|hit| hit.prefix_len)
                            .unwrap_or_default(),
                        first_table_hit_suffix_len: first_table_hit
                            .as_ref()
                            .map(|hit| hit.suffix_len)
                            .unwrap_or_default(),
                        first_table_hit_total_len: first_table_hit
                            .as_ref()
                            .map(|hit| hit.total_len)
                            .unwrap_or_default(),
                        gain_vs_scramble: gain,
                        ratio_vs_scramble: ratio,
                        scramble_script: if options.include_scripts {
                            puzzle.moves_text(&scramble)
                        } else {
                            String::new()
                        },
                        raw_solution_script: if options.include_scripts {
                            puzzle.moves_text(&result.raw_moves)
                        } else {
                            String::new()
                        },
                        optimized_solution_script: if options.include_scripts {
                            puzzle.moves_text(&result.optimized_moves)
                        } else {
                            String::new()
                        },
                    };

                    println!(
                        "{layout}-{difficulty} scramble={scramble_len} iter={} found={} method={} target={} ops={} raw={} opt={} android={} time={}ms nodes={} table_hit={} hit_depth={} hit_suffix={} reason={}",
                        iteration + 1,
                        record.found,
                        record.method.label(),
                        record.target_used,
                        record.operation_profile_used,
                        record.raw_solution_len,
                        record.optimized_solution_len,
                        record.android_solved,
                        record.elapsed_ms,
                        record.nodes,
                        record.first_table_hit,
                        record.first_table_hit_depth,
                        record.first_table_hit_suffix_len,
                        record.reason
                    );

                    records.push(record);
                }
            }
        }
    }

    write_csv(&csv_path, &records)?;
    write_markdown(&md_path, &options, &records)?;

    println!("CSV: {}", csv_path.display());
    println!("MD:  {}", md_path.display());
    Ok(())
}

fn run_ariadne_check(options: BenchOptions) -> io::Result<()> {
    fs::create_dir_all(&options.out_dir)?;

    let stamp = unix_stamp();
    let csv_path = options.out_dir.join(format!("ariadne_check_{stamp}.csv"));
    let md_path = options.out_dir.join(format!("ariadne_check_{stamp}.md"));
    let mut records = Vec::new();

    println!(
        "ariadne-check scramble_profile={} layouts={} difficulties={} scrambles={:?} iterations={} iteration_start={} seed={}",
        options.scramble_profile.label(),
        join_display(&options.layouts),
        join_display(&options.difficulties),
        options.scramble_lengths,
        options.iterations,
        options.iteration_start + 1,
        options.seed
    );

    for &layout in &options.layouts {
        for &difficulty in &options.difficulties {
            let puzzle = Puzzle::new(layout, difficulty);
            for &scramble_len in &options.scramble_lengths {
                for iteration_offset in 0..options.iterations {
                    let iteration = options.iteration_start + iteration_offset;
                    let run_seed =
                        derive_seed(options.seed, layout, difficulty, scramble_len, iteration);
                    let scramble = generate_scramble(
                        &puzzle,
                        scramble_len,
                        run_seed,
                        options.scramble_profile,
                        options.avoid_same_tape,
                    );
                    let ariadne_plan = ariadne_reduced_plan(&puzzle, &scramble);
                    let ariadne_solution = ariadne_solution_moves(&puzzle, &scramble);

                    let mut colors = puzzle.solved_colors.clone();
                    puzzle.apply_moves(&mut colors, &scramble);
                    puzzle.apply_moves(&mut colors, &ariadne_solution);

                    let exact_solved = colors == puzzle.solved_colors;
                    let uniform_solved = is_uniform_solved(&colors, &puzzle.face_indexes);
                    let android_solved =
                        is_android_solved(&colors, &puzzle.face_indexes, difficulty);

                    let record = AriadneCheckRecord {
                        layout,
                        difficulty,
                        scramble_len,
                        iteration: iteration + 1,
                        seed: run_seed,
                        ariadne_stack_len: ariadne_plan.len(),
                        ariadne_unit_len: ariadne_solution.len(),
                        exact_solved,
                        uniform_solved,
                        android_solved,
                        scramble_script: if options.include_scripts {
                            puzzle.moves_text(&scramble)
                        } else {
                            String::new()
                        },
                        ariadne_stack_script: if options.include_scripts {
                            ariadne_plan_text(&puzzle, &ariadne_plan)
                        } else {
                            String::new()
                        },
                        ariadne_solution_script: if options.include_scripts {
                            puzzle.moves_text(&ariadne_solution)
                        } else {
                            String::new()
                        },
                    };

                    println!(
                        "{layout}-{difficulty} scramble={scramble_len} iter={} stack={} unit={} exact={} android={}",
                        record.iteration,
                        record.ariadne_stack_len,
                        record.ariadne_unit_len,
                        record.exact_solved,
                        record.android_solved
                    );
                    records.push(record);
                }
            }
        }
    }

    write_ariadne_check_csv(&csv_path, &records)?;
    write_ariadne_check_markdown(&md_path, &options, &records)?;
    println!("wrote {}", csv_path.display());
    println!("wrote {}", md_path.display());
    Ok(())
}

fn run_heuristic_audit(options: BenchOptions) -> io::Result<()> {
    fs::create_dir_all(&options.out_dir)?;

    let stamp = unix_stamp();
    let csv_path = options.out_dir.join(format!("heuristic_audit_{stamp}.csv"));
    let md_path = options.out_dir.join(format!("heuristic_audit_{stamp}.md"));
    let mut records = Vec::new();

    println!(
        "heuristic-audit target={} layouts={} difficulties={} depth={} max_nodes={} time_limit_ms={}",
        options.solver.target_mode.label(),
        join_display(&options.layouts),
        join_display(&options.difficulties),
        options.solver.pattern_db_depth,
        options.solver.max_nodes,
        options.solver.time_limit_ms
    );

    for &layout in &options.layouts {
        for &difficulty in &options.difficulties {
            let puzzle = Puzzle::new(layout, difficulty);
            for target_mode in target_variants(options.solver.target_mode) {
                let mut solver = options.solver.clone();
                solver.target_mode = target_mode;
                let record = audit_puzzle_heuristics(&puzzle, &solver);
                println!(
                    "{}-{} target={} depth={} states={} hist={} canon={} hist_ratio={:.4} canon_ratio={:.4} pearson={:.4} time={}ms reason={}",
                    record.layout,
                    record.difficulty,
                    record.target.label(),
                    record.depth,
                    record.states,
                    record.histogram_keys,
                    record.canonical_histogram_keys,
                    ratio(record.histogram_keys, record.states),
                    ratio(record.canonical_histogram_keys, record.states),
                    record.pearson_score_distance,
                    record.elapsed_ms,
                    record.reason
                );
                records.push(record);
            }
        }
    }

    write_heuristic_audit_csv(&csv_path, &records)?;
    write_heuristic_audit_markdown(&md_path, &options, &records)?;
    println!("CSV: {}", csv_path.display());
    println!("MD:  {}", md_path.display());
    Ok(())
}

fn run_projection_audit(options: BenchOptions) -> io::Result<()> {
    fs::create_dir_all(&options.out_dir)?;

    let stamp = unix_stamp();
    let csv_path = options
        .out_dir
        .join(format!("projection_audit_{stamp}.csv"));
    let md_path = options.out_dir.join(format!("projection_audit_{stamp}.md"));
    let mut records = Vec::new();

    println!(
        "projection-audit target={} layouts={} difficulties={} depth={} max_nodes={} time_limit_ms={}",
        options.solver.target_mode.label(),
        join_display(&options.layouts),
        join_display(&options.difficulties),
        options.solver.pattern_db_depth,
        options.solver.max_nodes,
        options.solver.time_limit_ms
    );

    for &layout in &options.layouts {
        for &difficulty in &options.difficulties {
            let puzzle = Puzzle::new(layout, difficulty);
            for target_mode in target_variants(options.solver.target_mode) {
                let mut solver = options.solver.clone();
                solver.target_mode = target_mode;
                for &projection in &options.projection_kinds {
                    let record = audit_puzzle_projection(&puzzle, &solver, projection);
                    println!(
                        "{}-{} target={} projection={} depth={} states={} keys={} ratio={:.4} mean_span={:.2} p95_span={} max_span={} mean_bucket={:.2} p95_bucket={} max_bucket={} time={}ms reason={}",
                        record.layout,
                        record.difficulty,
                        record.target.label(),
                        record.projection.label(),
                        record.depth,
                        record.states,
                        record.keys,
                        ratio(record.keys, record.states),
                        record.mean_depth_span,
                        record.p95_depth_span,
                        record.max_depth_span,
                        record.mean_states_per_key,
                        record.p95_states_per_key,
                        record.max_states_per_key,
                        record.elapsed_ms,
                        record.reason
                    );
                    records.push(record);
                }
            }
        }
    }

    write_projection_audit_csv(&csv_path, &records)?;
    write_projection_audit_markdown(&md_path, &options, &records)?;
    println!("CSV: {}", csv_path.display());
    println!("MD:  {}", md_path.display());
    Ok(())
}

fn run_axis_ring_pdb_audit(options: BenchOptions) -> io::Result<()> {
    fs::create_dir_all(&options.out_dir)?;

    let stamp = unix_stamp();
    let csv_path = options
        .out_dir
        .join(format!("axis_ring_pdb_audit_{stamp}.csv"));
    let md_path = options
        .out_dir
        .join(format!("axis_ring_pdb_audit_{stamp}.md"));
    let mut records = Vec::new();
    let iterations = iteration_values(&options);
    let target_mode = restricted_seed_target(options.solver.target_mode);
    let axis_table_depth = options.axis_ring_rescue_table_depth;
    let target_expand_depth = options.axis_ring_rescue_expand_depth;
    let pdb_depth = options.solver.pattern_db_depth;

    println!(
        "axis-ring-pdb-audit source={} target={} scramble_profile={} layouts={} difficulties={} scrambles={:?} iterations={} seed={} seed_count={} seed_steps={}-{} random_walk={}-{} axis_table_depth={} target_expand_depth={} pdb_depth={} max_nodes={} time_limit_ms={}",
        options.pdb_seed_source.label(),
        target_mode.label(),
        options.scramble_profile.label(),
        join_display(&options.layouts),
        join_display(&options.difficulties),
        options.scramble_lengths,
        iterations.len(),
        options.seed,
        options.pdb_seed_count,
        options.pdb_seed_step_start,
        options.pdb_seed_step_end,
        options.pdb_random_walk_min,
        options.pdb_random_walk_max,
        axis_table_depth,
        target_expand_depth,
        pdb_depth,
        options.solver.max_nodes,
        options.solver.time_limit_ms
    );

    for &layout in &options.layouts {
        for &difficulty in &options.difficulties {
            let puzzle = Puzzle::new(layout, difficulty);
            let pdb = build_multi_start_pdb(&puzzle, &options, target_mode, pdb_depth);
            println!(
                "prepared {}-{} source={} pdb target_states={} pdb_states={} depth_counts={} nodes={} elapsed={}ms truncated={} reason={}",
                layout,
                difficulty,
                options.pdb_seed_source.label(),
                pdb.target_states,
                pdb.distances.len(),
                join_usize_counts(&pdb.depth_counts),
                pdb.nodes,
                pdb.elapsed_ms,
                pdb.truncated,
                pdb.reason
            );

            for &scramble_len in &options.scramble_lengths {
                for &iteration in &iterations {
                    let seed =
                        derive_seed(options.seed, layout, difficulty, scramble_len, iteration);
                    let scramble = generate_scramble(
                        &puzzle,
                        scramble_len,
                        seed,
                        options.scramble_profile,
                        options.avoid_same_tape,
                    );
                    let record = evaluate_axis_ring_pdb_on_ariadne_path(
                        &puzzle,
                        target_mode,
                        scramble_len,
                        iteration,
                        seed,
                        axis_table_depth,
                        target_expand_depth,
                        pdb_depth,
                        &pdb,
                        options.pdb_seed_source,
                        &scramble,
                    );
                    println!(
                        "{}-{} scramble={} iter={} ariadne={} hits={}/{} first_hit={} first_remaining={} best_prefix={} est_total={} start_hit={} pearson={:.3}",
                        layout,
                        difficulty,
                        scramble_len,
                        iteration + 1,
                        record.ariadne_unit_len,
                        record.hit_count,
                        record.path_states,
                        fmt_opt_usize(record.first_hit_step),
                        fmt_opt_usize(record.first_hit_remaining_to_solved),
                        fmt_opt_usize(record.best_prefix_to_axis_ring),
                        fmt_opt_usize(record.estimated_total_with_max_suffix),
                        fmt_opt_u8(record.start_hit_distance),
                        record.path_distance_remaining_pearson
                    );
                    records.push(record);
                }
            }
        }
    }

    write_axis_ring_pdb_audit_csv(&csv_path, &records)?;
    write_axis_ring_pdb_audit_markdown(&md_path, &options, &records)?;
    println!("CSV: {}", csv_path.display());
    println!("MD:  {}", md_path.display());
    Ok(())
}

fn run_move_delta_audit(options: BenchOptions) -> io::Result<()> {
    fs::create_dir_all(&options.out_dir)?;

    let stamp = unix_stamp();
    let csv_path = options
        .out_dir
        .join(format!("move_delta_audit_{stamp}.csv"));
    let md_path = options.out_dir.join(format!("move_delta_audit_{stamp}.md"));
    let mut records = Vec::new();
    let iterations = iteration_values(&options);

    println!(
        "move-delta-audit target={} operation_profile={} scramble_profile={} layouts={} difficulties={} scrambles={:?} iterations={} seed={}",
        options.solver.target_mode.label(),
        options.solver.operation_profile.label(),
        options.scramble_profile.label(),
        join_display(&options.layouts),
        join_display(&options.difficulties),
        options.scramble_lengths,
        iterations.len(),
        options.seed
    );

    for &layout in &options.layouts {
        for &difficulty in &options.difficulties {
            let puzzle = Puzzle::new(layout, difficulty);
            let operation_profile = options.solver.operation_profile.for_layout(layout);
            let operations = build_operations(&puzzle, operation_profile);
            println!(
                "prepared {}-{} operations={} profile={}",
                layout,
                difficulty,
                operations.len(),
                operation_profile.label()
            );

            for &scramble_len in &options.scramble_lengths {
                for &iteration in &iterations {
                    let seed =
                        derive_seed(options.seed, layout, difficulty, scramble_len, iteration);
                    let scramble = generate_scramble(
                        &puzzle,
                        scramble_len,
                        seed,
                        options.scramble_profile,
                        options.avoid_same_tape,
                    );
                    let ariadne_solution = ariadne_solution_moves(&puzzle, &scramble);
                    let mut colors = puzzle.solved_colors.clone();
                    puzzle.apply_moves(&mut colors, &scramble);

                    let start_len = records.len();
                    for (step, &ariadne_move) in ariadne_solution.iter().enumerate() {
                        records.extend(evaluate_move_delta_step(
                            &puzzle,
                            &options.solver,
                            operation_profile,
                            &operations,
                            scramble_len,
                            iteration,
                            seed,
                            ariadne_solution.len(),
                            step,
                            ariadne_move,
                            &colors,
                        ));
                        puzzle.apply_move(&mut colors, ariadne_move);
                    }
                    let added = records.len() - start_len;
                    println!(
                        "{}-{} scramble={} iter={} ariadne={} rows={}",
                        layout,
                        difficulty,
                        scramble_len,
                        iteration + 1,
                        ariadne_solution.len(),
                        added
                    );
                }
            }
        }
    }

    write_move_delta_audit_csv(&csv_path, &records)?;
    write_move_delta_audit_markdown(&md_path, &options, &records)?;
    println!("CSV: {}", csv_path.display());
    println!("MD:  {}", md_path.display());
    Ok(())
}

fn run_feature_cost_audit(options: BenchOptions) -> io::Result<()> {
    fs::create_dir_all(&options.out_dir)?;

    let stamp = unix_stamp();
    let csv_path = options
        .out_dir
        .join(format!("feature_cost_audit_{stamp}.csv"));
    let md_path = options
        .out_dir
        .join(format!("feature_cost_audit_{stamp}.md"));
    let mut records = Vec::new();
    let iterations = iteration_values(&options);
    let repeats = options.feature_repeats;

    println!(
        "feature-cost-audit target={} scramble_profile={} layouts={} difficulties={} scrambles={:?} states={} repeats={} seed={}",
        options.solver.target_mode.label(),
        options.scramble_profile.label(),
        join_display(&options.layouts),
        join_display(&options.difficulties),
        options.scramble_lengths,
        iterations.len(),
        repeats,
        options.seed
    );

    for &layout in &options.layouts {
        for &difficulty in &options.difficulties {
            let puzzle = Puzzle::new(layout, difficulty);
            for &scramble_len in &options.scramble_lengths {
                let mut states = Vec::with_capacity(iterations.len());
                for &iteration in &iterations {
                    let seed =
                        derive_seed(options.seed, layout, difficulty, scramble_len, iteration);
                    let scramble = generate_scramble(
                        &puzzle,
                        scramble_len,
                        seed,
                        options.scramble_profile,
                        options.avoid_same_tape,
                    );
                    let mut colors = puzzle.solved_colors.clone();
                    puzzle.apply_moves(&mut colors, &scramble);
                    states.push(colors);
                }

                let mut add_record = |feature: &'static str, total_ns: u128, checksum: u64| {
                    let evals = states.len() * repeats;
                    let mean_ns = if evals == 0 {
                        0.0
                    } else {
                        total_ns as f64 / evals as f64
                    };
                    println!(
                        "{}-{} scramble={} feature={} evals={} mean_ns={:.1} checksum={}",
                        layout, difficulty, scramble_len, feature, evals, mean_ns, checksum
                    );
                    records.push(FeatureCostRecord {
                        layout,
                        difficulty,
                        scramble_len,
                        states: states.len(),
                        repeats,
                        feature,
                        evals,
                        total_ns,
                        mean_ns,
                        checksum,
                    });
                };

                let (total_ns, checksum) = benchmark_feature_cost(&states, repeats, |colors| {
                    score_state(
                        colors,
                        &puzzle,
                        options.solver.target_mode,
                        options.solver.region_pair_weight,
                        None,
                    ) as i64 as u64
                });
                add_record("score_state", total_ns, checksum);

                let (total_ns, checksum) = benchmark_feature_cost(&states, repeats, |colors| {
                    ring_rotation_miss_total(&puzzle, colors) as u64
                });
                add_record("ring_rotation_miss", total_ns, checksum);

                let (total_ns, checksum) = benchmark_feature_cost(&states, repeats, |colors| {
                    let metrics = permutation_matching_metrics(&puzzle, colors);
                    (metrics.slot_cost as u64)
                        .wrapping_mul(1_000_003)
                        .wrapping_add(metrics.cycle_cost as u64)
                });
                add_record("permutation_matching", total_ns, checksum);

                let (total_ns, checksum) = benchmark_feature_cost(&states, repeats, |colors| {
                    (ring_entropy_total(&puzzle, colors) * 1_000_000.0).round() as u64
                });
                add_record("ring_entropy", total_ns, checksum);

                let (total_ns, checksum) = benchmark_feature_cost(&states, repeats, |colors| {
                    face_pair_home_matches_key(&puzzle, colors)
                        .iter()
                        .fold(0_u64, |acc, &value| acc + u64::from(value))
                });
                add_record("axis_pair_quality", total_ns, checksum);

                let (total_ns, checksum) = benchmark_feature_cost(&states, repeats, |colors| {
                    tape_segment_conflicts(&puzzle, colors) as u64
                });
                add_record("tape_segment_conflicts", total_ns, checksum);
            }
        }
    }

    write_feature_cost_csv(&csv_path, &records)?;
    write_feature_cost_markdown(&md_path, &options, &records)?;
    println!("CSV: {}", csv_path.display());
    println!("MD:  {}", md_path.display());
    Ok(())
}

fn benchmark_feature_cost<F>(states: &[Vec<Color>], repeats: usize, mut feature: F) -> (u128, u64)
where
    F: FnMut(&[Color]) -> u64,
{
    let started = Instant::now();
    let mut checksum = 0_u64;
    for _ in 0..repeats {
        for colors in states {
            checksum = checksum.wrapping_add(black_box(feature(black_box(colors.as_slice()))));
        }
    }
    (started.elapsed().as_nanos(), checksum)
}

fn run_commutator_scan(options: BenchOptions) -> io::Result<()> {
    fs::create_dir_all(&options.out_dir)?;

    let stamp = unix_stamp();
    let csv_path = options.out_dir.join(format!("commutator_scan_{stamp}.csv"));
    let catalog_path = options
        .out_dir
        .join(format!("commutator_catalog_{stamp}.csv"));
    let md_path = options.out_dir.join(format!("commutator_scan_{stamp}.md"));
    let mut records = Vec::new();

    println!(
        "commutator-scan layouts={} difficulties={} max_len={} top={} max_nodes={} time_limit_ms={}",
        join_display(&options.layouts),
        join_display(&options.difficulties),
        options.commutator_max_len,
        options.commutator_top,
        options.solver.max_nodes,
        options.solver.time_limit_ms
    );

    for &layout in &options.layouts {
        for &difficulty in &options.difficulties {
            let puzzle = Puzzle::new(layout, difficulty);
            let record = scan_commutators(&puzzle, &options);
            println!(
                "{}-{} max_len={} sequences={} pairs={} unique={} min_support={} clean3={} double2x2={} truncated={} elapsed={}ms",
                record.layout,
                record.difficulty,
                record.max_len,
                record.sequences,
                record.pairs_examined,
                record.unique_permutations,
                record.min_support,
                record.clean_3_cycles,
                record.double_transpositions,
                record.truncated,
                record.elapsed_ms
            );
            records.push(record);
        }
    }

    write_commutator_scan_csv(&csv_path, &records)?;
    write_commutator_catalog_csv(&catalog_path, &records)?;
    write_commutator_scan_markdown(&md_path, &options, &records)?;
    println!("CSV: {}", csv_path.display());
    println!("Catalog CSV: {}", catalog_path.display());
    println!("MD:  {}", md_path.display());
    Ok(())
}

fn run_commutator_applicability_audit(options: BenchOptions) -> io::Result<()> {
    fs::create_dir_all(&options.out_dir)?;

    let stamp = unix_stamp();
    let csv_path = options
        .out_dir
        .join(format!("commutator_applicability_audit_{stamp}.csv"));
    let md_path = options
        .out_dir
        .join(format!("commutator_applicability_audit_{stamp}.md"));
    let mut records = Vec::new();

    println!(
        "commutator-applicability-audit target={} scramble_profile={} layouts={} difficulties={} scrambles={:?} iterations={} seed={} max_len={}",
        options.solver.target_mode.label(),
        options.scramble_profile.label(),
        join_display(&options.layouts),
        join_display(&options.difficulties),
        options.scramble_lengths,
        iteration_label(&options),
        options.seed,
        options.commutator_max_len
    );

    let iterations = iteration_values(&options);
    for &layout in &options.layouts {
        for &difficulty in &options.difficulties {
            let puzzle = Puzzle::new(layout, difficulty);
            let scan_record = scan_commutators(&puzzle, &options);
            let primitives = commutator_primitives_from_scan(&puzzle, &scan_record, false);
            let distinct_triples = distinct_commutator_triples(&primitives);
            let covered_positions = covered_commutator_positions(&primitives);
            println!(
                "prepared {}-{} clean3={} distinct_triples={} covered_positions={} scan_elapsed={}ms",
                layout,
                difficulty,
                primitives.len(),
                distinct_triples,
                covered_positions,
                scan_record.elapsed_ms
            );

            for &scramble_len in &options.scramble_lengths {
                for &iteration in &iterations {
                    let run_seed =
                        derive_seed(options.seed, layout, difficulty, scramble_len, iteration);
                    let scramble = generate_scramble(
                        &puzzle,
                        scramble_len,
                        run_seed,
                        options.scramble_profile,
                        options.avoid_same_tape,
                    );
                    let mut colors = puzzle.solved_colors.clone();
                    puzzle.apply_moves(&mut colors, &scramble);

                    let target =
                        best_commutator_target(&puzzle, &colors, options.solver.target_mode);
                    let record = audit_commutator_applicability(
                        &puzzle,
                        &colors,
                        &target,
                        &primitives,
                        layout,
                        difficulty,
                        options.solver.target_mode,
                        scramble_len,
                        iteration + 1,
                        run_seed,
                        distinct_triples,
                        covered_positions,
                    );
                    println!(
                        "{}-{} scramble={} iter={} mismatches={} improving={} strong={} best_delta={} best_after={}",
                        layout,
                        difficulty,
                        scramble_len,
                        iteration + 1,
                        record.initial_mismatches,
                        record.improving_commutators,
                        record.strong_commutators,
                        record.best_delta,
                        record.best_after_mismatches
                    );
                    records.push(record);
                }
            }
        }
    }

    write_commutator_applicability_csv(&csv_path, &records)?;
    write_commutator_applicability_markdown(&md_path, &options, &records)?;
    println!("CSV: {}", csv_path.display());
    println!("MD:  {}", md_path.display());
    Ok(())
}

fn run_commutator_greedy_audit(options: BenchOptions) -> io::Result<()> {
    fs::create_dir_all(&options.out_dir)?;

    let stamp = unix_stamp();
    let csv_path = options
        .out_dir
        .join(format!("commutator_greedy_audit_{stamp}.csv"));
    let md_path = options
        .out_dir
        .join(format!("commutator_greedy_audit_{stamp}.md"));
    let mut records = Vec::new();

    println!(
        "commutator-greedy-audit target={} dynamic_target={} plateau_lookahead={} suffix_rescue={} suffix_time_limit_ms={} scramble_profile={} layouts={} difficulties={} scrambles={:?} iterations={} seed={} max_len={} greedy_steps={}",
        options.solver.target_mode.label(),
        options.commutator_dynamic_target,
        options.commutator_plateau_lookahead,
        options.commutator_suffix_rescue,
        options.commutator_suffix_time_limit_ms,
        options.scramble_profile.label(),
        join_display(&options.layouts),
        join_display(&options.difficulties),
        options.scramble_lengths,
        iteration_label(&options),
        options.seed,
        options.commutator_max_len,
        options.commutator_greedy_steps
    );

    let iterations = iteration_values(&options);
    for &layout in &options.layouts {
        for &difficulty in &options.difficulties {
            let puzzle = Puzzle::new(layout, difficulty);
            let scan_record = scan_commutators(&puzzle, &options);
            let primitives = commutator_primitives_from_scan(&puzzle, &scan_record, true);
            let dynamic_targets = commutator_targets(&puzzle, options.solver.target_mode);
            let suffix_solver = if options.commutator_suffix_rescue {
                let suffix_config = commutator_suffix_solver_config(
                    &options.solver,
                    options.commutator_suffix_time_limit_ms,
                );
                let suffix_artifacts = prepare_solver(&puzzle, &suffix_config)?;
                Some((suffix_config, suffix_artifacts))
            } else {
                None
            };
            println!(
                "prepared {}-{} primitives={} scan_elapsed={}ms",
                layout,
                difficulty,
                primitives.len(),
                scan_record.elapsed_ms
            );

            for &scramble_len in &options.scramble_lengths {
                for &iteration in &iterations {
                    let run_seed =
                        derive_seed(options.seed, layout, difficulty, scramble_len, iteration);
                    let scramble = generate_scramble(
                        &puzzle,
                        scramble_len,
                        run_seed,
                        options.scramble_profile,
                        options.avoid_same_tape,
                    );
                    let ariadne_solution_len = ariadne_solution_len(&puzzle, &scramble);
                    let mut scrambled_colors = puzzle.solved_colors.clone();
                    puzzle.apply_moves(&mut scrambled_colors, &scramble);
                    let static_target = vec![best_commutator_target(
                        &puzzle,
                        &scrambled_colors,
                        options.solver.target_mode,
                    )];
                    let greedy_targets = if options.commutator_dynamic_target {
                        dynamic_targets.as_slice()
                    } else {
                        static_target.as_slice()
                    };

                    let started = Instant::now();
                    let greedy = greedy_commutator_solve(
                        &puzzle,
                        &scrambled_colors,
                        greedy_targets,
                        &primitives,
                        options.commutator_greedy_steps,
                        options.commutator_plateau_lookahead,
                    );
                    let greedy_optimized_moves = if options.solver.optimize {
                        optimize_solution(&puzzle, &scrambled_colors, &greedy.moves)
                    } else {
                        greedy.moves.clone()
                    };
                    let mut plateau_colors = scrambled_colors.clone();
                    puzzle.apply_moves(&mut plateau_colors, &greedy.moves);
                    let greedy_final_mismatches =
                        best_mismatches_to_targets(&plateau_colors, greedy_targets);

                    let mut raw_moves = greedy.moves.clone();
                    let mut suffix_attempted = false;
                    let mut suffix_found = false;
                    let mut suffix_raw_len = 0usize;
                    let mut suffix_optimized_len = 0usize;
                    let mut suffix_reason = String::new();
                    if greedy_final_mismatches > 0 {
                        if let Some((suffix_config, suffix_artifacts)) = &suffix_solver {
                            suffix_attempted = true;
                            let suffix_result = solve_puzzle(
                                &puzzle,
                                &plateau_colors,
                                suffix_config,
                                suffix_artifacts,
                            );
                            suffix_raw_len = suffix_result.raw_moves.len();
                            suffix_optimized_len = suffix_result.optimized_moves.len();
                            suffix_reason = suffix_result.reason.clone();
                            if suffix_result.found {
                                suffix_found = true;
                                raw_moves.extend(suffix_result.optimized_moves);
                            }
                        }
                    }

                    let optimized_moves = if options.solver.optimize {
                        optimize_solution(&puzzle, &scrambled_colors, &raw_moves)
                    } else {
                        raw_moves.clone()
                    };
                    let mut optimized_colors = scrambled_colors.clone();
                    puzzle.apply_moves(&mut optimized_colors, &optimized_moves);
                    let found = is_target_solved(
                        &optimized_colors,
                        &puzzle,
                        acceptance_target(options.solver.target_mode),
                    );
                    let final_mismatches =
                        best_mismatches_to_targets(&optimized_colors, &dynamic_targets);
                    let reason = if found {
                        if suffix_found {
                            "found_suffix_rescue".to_string()
                        } else {
                            "found".to_string()
                        }
                    } else if suffix_attempted {
                        format!("{}+suffix:{}", greedy.reason, suffix_reason)
                    } else {
                        greedy.reason.clone()
                    };
                    let mean_step_delta = if greedy.deltas.is_empty() {
                        0.0
                    } else {
                        greedy.deltas.iter().sum::<isize>() as f64 / greedy.deltas.len() as f64
                    };
                    let best_step_delta = greedy.deltas.iter().copied().max().unwrap_or(0);
                    let record = CommutatorGreedyRecord {
                        layout,
                        difficulty,
                        target_mode: options.solver.target_mode,
                        scramble_len,
                        iteration: iteration + 1,
                        seed: run_seed,
                        ariadne_solution_len,
                        initial_mismatches: greedy.initial_mismatches,
                        final_mismatches,
                        found,
                        reason,
                        commutator_steps: greedy.steps,
                        greedy_raw_len: greedy.moves.len(),
                        greedy_optimized_len: greedy_optimized_moves.len(),
                        suffix_attempted,
                        suffix_found,
                        suffix_raw_len,
                        suffix_optimized_len,
                        suffix_reason,
                        raw_solution_len: raw_moves.len(),
                        optimized_solution_len: optimized_moves.len(),
                        best_step_delta,
                        mean_step_delta,
                        elapsed_ms: started.elapsed().as_millis(),
                        raw_solution_script: if options.include_scripts {
                            puzzle.moves_text(&greedy.moves)
                        } else {
                            String::new()
                        },
                        optimized_solution_script: if options.include_scripts {
                            puzzle.moves_text(&optimized_moves)
                        } else {
                            String::new()
                        },
                    };
                    println!(
                        "{}-{} scramble={} iter={} found={} steps={} raw={} opt={} final_mismatches={} reason={}",
                        layout,
                        difficulty,
                        scramble_len,
                        iteration + 1,
                        record.found,
                        record.commutator_steps,
                        record.raw_solution_len,
                        record.optimized_solution_len,
                        record.final_mismatches,
                        record.reason
                    );
                    records.push(record);
                }
            }
        }
    }

    write_commutator_greedy_csv(&csv_path, &records)?;
    write_commutator_greedy_markdown(&md_path, &options, &records)?;
    println!("CSV: {}", csv_path.display());
    println!("MD:  {}", md_path.display());
    Ok(())
}

fn run_commutator_plateau_audit(options: BenchOptions) -> io::Result<()> {
    fs::create_dir_all(&options.out_dir)?;

    let stamp = unix_stamp();
    let csv_path = options
        .out_dir
        .join(format!("commutator_plateau_audit_{stamp}.csv"));
    let md_path = options
        .out_dir
        .join(format!("commutator_plateau_audit_{stamp}.md"));
    let mut records = Vec::new();

    println!(
        "commutator-plateau-audit target={} dynamic_target={} plateau_lookahead={} scramble_profile={} layouts={} difficulties={} scrambles={:?} iterations={} seed={} max_len={} greedy_steps={}",
        options.solver.target_mode.label(),
        options.commutator_dynamic_target,
        options.commutator_plateau_lookahead,
        options.scramble_profile.label(),
        join_display(&options.layouts),
        join_display(&options.difficulties),
        options.scramble_lengths,
        iteration_label(&options),
        options.seed,
        options.commutator_max_len,
        options.commutator_greedy_steps
    );

    let iterations = iteration_values(&options);
    for &layout in &options.layouts {
        for &difficulty in &options.difficulties {
            let puzzle = Puzzle::new(layout, difficulty);
            let scan_record = scan_commutators(&puzzle, &options);
            let primitives = commutator_primitives_from_scan(&puzzle, &scan_record, true);
            let dynamic_targets = commutator_targets(&puzzle, options.solver.target_mode);
            println!(
                "prepared {}-{} primitives={} scan_elapsed={}ms",
                layout,
                difficulty,
                primitives.len(),
                scan_record.elapsed_ms
            );

            for &scramble_len in &options.scramble_lengths {
                for &iteration in &iterations {
                    let run_seed =
                        derive_seed(options.seed, layout, difficulty, scramble_len, iteration);
                    let scramble = generate_scramble(
                        &puzzle,
                        scramble_len,
                        run_seed,
                        options.scramble_profile,
                        options.avoid_same_tape,
                    );
                    let mut scrambled_colors = puzzle.solved_colors.clone();
                    puzzle.apply_moves(&mut scrambled_colors, &scramble);
                    let static_target = vec![best_commutator_target(
                        &puzzle,
                        &scrambled_colors,
                        options.solver.target_mode,
                    )];
                    let greedy_targets = if options.commutator_dynamic_target {
                        dynamic_targets.as_slice()
                    } else {
                        static_target.as_slice()
                    };

                    let greedy = greedy_commutator_solve(
                        &puzzle,
                        &scrambled_colors,
                        greedy_targets,
                        &primitives,
                        options.commutator_greedy_steps,
                        options.commutator_plateau_lookahead,
                    );
                    let mut plateau_colors = scrambled_colors.clone();
                    puzzle.apply_moves(&mut plateau_colors, &greedy.moves);
                    let audit_target = best_commutator_target(
                        &puzzle,
                        &plateau_colors,
                        options.solver.target_mode,
                    );
                    let record = audit_commutator_plateau(
                        &puzzle,
                        &plateau_colors,
                        &audit_target,
                        &dynamic_targets,
                        &primitives,
                        layout,
                        difficulty,
                        options.solver.target_mode,
                        scramble_len,
                        iteration + 1,
                        run_seed,
                        &greedy,
                    );
                    println!(
                        "{}-{} scramble={} iter={} plateau={} best_dynamic={} parity={} flex={} direct_best_delta={} touch=({},{},{},{})",
                        layout,
                        difficulty,
                        scramble_len,
                        iteration + 1,
                        record.plateau_mismatches,
                        record.best_dynamic_mismatches,
                        record.residual_canonical_parity,
                        record.residual_flex_parity,
                        record.direct_best_delta,
                        record.support_touch_1,
                        record.support_touch_2,
                        record.support_touch_3,
                        record.support_touch_4
                    );
                    records.push(record);
                }
            }
        }
    }

    write_commutator_plateau_csv(&csv_path, &records)?;
    write_commutator_plateau_markdown(&md_path, &options, &records)?;
    println!("CSV: {}", csv_path.display());
    println!("MD:  {}", md_path.display());
    Ok(())
}

fn run_commutator_endgame_audit(options: BenchOptions) -> io::Result<()> {
    fs::create_dir_all(&options.out_dir)?;

    let stamp = unix_stamp();
    let csv_path = options
        .out_dir
        .join(format!("commutator_endgame_audit_{stamp}.csv"));
    let md_path = options
        .out_dir
        .join(format!("commutator_endgame_audit_{stamp}.md"));
    let mut records = Vec::new();

    println!(
        "commutator-endgame-audit target={} dynamic_target={} plateau_lookahead={} endgame_depth={} endgame_width={} endgame_time_limit_ms={} scramble_profile={} layouts={} difficulties={} scrambles={:?} iterations={} seed={} max_len={} greedy_steps={}",
        options.solver.target_mode.label(),
        options.commutator_dynamic_target,
        options.commutator_plateau_lookahead,
        options.commutator_endgame_depth,
        options.commutator_endgame_width,
        options.commutator_endgame_time_limit_ms,
        options.scramble_profile.label(),
        join_display(&options.layouts),
        join_display(&options.difficulties),
        options.scramble_lengths,
        iteration_label(&options),
        options.seed,
        options.commutator_max_len,
        options.commutator_greedy_steps
    );

    let iterations = iteration_values(&options);
    for &layout in &options.layouts {
        for &difficulty in &options.difficulties {
            let puzzle = Puzzle::new(layout, difficulty);
            let scan_record = scan_commutators(&puzzle, &options);
            let primitives = commutator_primitives_from_scan(&puzzle, &scan_record, true);
            let dynamic_targets = commutator_targets(&puzzle, options.solver.target_mode);
            println!(
                "prepared {}-{} primitives={} scan_elapsed={}ms",
                layout,
                difficulty,
                primitives.len(),
                scan_record.elapsed_ms
            );

            for &scramble_len in &options.scramble_lengths {
                for &iteration in &iterations {
                    let run_seed =
                        derive_seed(options.seed, layout, difficulty, scramble_len, iteration);
                    let scramble = generate_scramble(
                        &puzzle,
                        scramble_len,
                        run_seed,
                        options.scramble_profile,
                        options.avoid_same_tape,
                    );
                    let ariadne_solution_len = ariadne_solution_len(&puzzle, &scramble);
                    let mut scrambled_colors = puzzle.solved_colors.clone();
                    puzzle.apply_moves(&mut scrambled_colors, &scramble);
                    let static_target = vec![best_commutator_target(
                        &puzzle,
                        &scrambled_colors,
                        options.solver.target_mode,
                    )];
                    let greedy_targets = if options.commutator_dynamic_target {
                        dynamic_targets.as_slice()
                    } else {
                        static_target.as_slice()
                    };

                    let started = Instant::now();
                    let greedy = greedy_commutator_solve(
                        &puzzle,
                        &scrambled_colors,
                        greedy_targets,
                        &primitives,
                        options.commutator_greedy_steps,
                        options.commutator_plateau_lookahead,
                    );
                    let mut plateau_colors = scrambled_colors.clone();
                    puzzle.apply_moves(&mut plateau_colors, &greedy.moves);
                    let plateau_mismatches =
                        best_mismatches_to_targets(&plateau_colors, greedy_targets);
                    let target = nearest_commutator_target(&plateau_colors, greedy_targets);
                    let mismatch_positions = mismatch_positions(&plateau_colors, target);

                    let endgame = if plateau_mismatches == 0 {
                        CommutatorEndgameSearchResult {
                            found: true,
                            reason: "already_solved".to_string(),
                            moves: Vec::new(),
                            depth: 0,
                            nodes: 0,
                            best_mismatches: 0,
                        }
                    } else {
                        commutator_endgame_search(
                            &puzzle,
                            &plateau_colors,
                            greedy_targets,
                            &primitives,
                            &options,
                        )
                    };

                    let mut raw_moves = greedy.moves.clone();
                    raw_moves.extend_from_slice(&endgame.moves);
                    let optimized_moves = if options.solver.optimize {
                        optimize_solution(&puzzle, &scrambled_colors, &raw_moves)
                    } else {
                        raw_moves.clone()
                    };
                    let mut optimized_colors = scrambled_colors.clone();
                    puzzle.apply_moves(&mut optimized_colors, &optimized_moves);
                    let found = is_target_solved(
                        &optimized_colors,
                        &puzzle,
                        acceptance_target(options.solver.target_mode),
                    );
                    let final_mismatches =
                        best_mismatches_to_targets(&optimized_colors, &dynamic_targets);
                    let reason = if found {
                        if endgame.reason == "already_solved" {
                            "found_greedy".to_string()
                        } else {
                            "found_endgame".to_string()
                        }
                    } else {
                        format!("{}+endgame:{}", greedy.reason, endgame.reason)
                    };

                    let record = CommutatorEndgameRecord {
                        layout,
                        difficulty,
                        target_mode: options.solver.target_mode,
                        scramble_len,
                        iteration: iteration + 1,
                        seed: run_seed,
                        ariadne_solution_len,
                        greedy_reason: greedy.reason.clone(),
                        greedy_steps: greedy.steps,
                        greedy_raw_len: greedy.moves.len(),
                        plateau_mismatches,
                        found,
                        reason,
                        endgame_found: endgame.found,
                        endgame_depth: endgame.depth,
                        endgame_nodes: endgame.nodes,
                        endgame_raw_len: endgame.moves.len(),
                        endgame_best_mismatches: endgame.best_mismatches,
                        total_raw_len: raw_moves.len(),
                        total_optimized_len: optimized_moves.len(),
                        final_mismatches,
                        elapsed_ms: started.elapsed().as_millis(),
                        mismatch_positions: join_usize(&mismatch_positions),
                        endgame_script: if options.include_scripts {
                            puzzle.moves_text(&endgame.moves)
                        } else {
                            String::new()
                        },
                        optimized_script: if options.include_scripts {
                            puzzle.moves_text(&optimized_moves)
                        } else {
                            String::new()
                        },
                    };
                    println!(
                        "{}-{} scramble={} iter={} found={} plateau={} endgame_depth={} endgame_raw={} opt={} final_mismatches={} nodes={} reason={}",
                        layout,
                        difficulty,
                        scramble_len,
                        iteration + 1,
                        record.found,
                        record.plateau_mismatches,
                        record.endgame_depth,
                        record.endgame_raw_len,
                        record.total_optimized_len,
                        record.final_mismatches,
                        record.endgame_nodes,
                        record.reason
                    );
                    records.push(record);
                }
            }
        }
    }

    write_commutator_endgame_csv(&csv_path, &records)?;
    write_commutator_endgame_markdown(&md_path, &options, &records)?;
    println!("CSV: {}", csv_path.display());
    println!("MD:  {}", md_path.display());
    Ok(())
}

fn run_commutator_decomposition_audit(options: BenchOptions) -> io::Result<()> {
    fs::create_dir_all(&options.out_dir)?;

    let stamp = unix_stamp();
    let csv_path = options
        .out_dir
        .join(format!("commutator_decomposition_audit_{stamp}.csv"));
    let md_path = options
        .out_dir
        .join(format!("commutator_decomposition_audit_{stamp}.md"));
    let mut records = Vec::new();

    println!(
        "commutator-decomposition-audit target={} unit_time_limit_ms={} scramble_profile={} layouts={} difficulties={} scrambles={:?} iterations={} seed={} max_len={} greedy_steps={}",
        options.solver.target_mode.label(),
        options.commutator_suffix_time_limit_ms,
        options.scramble_profile.label(),
        join_display(&options.layouts),
        join_display(&options.difficulties),
        options.scramble_lengths,
        iteration_label(&options),
        options.seed,
        options.commutator_max_len,
        options.commutator_greedy_steps
    );

    let iterations = iteration_values(&options);
    for &layout in &options.layouts {
        for &difficulty in &options.difficulties {
            let puzzle = Puzzle::new(layout, difficulty);
            let scan_record = scan_commutators(&puzzle, &options);
            let primitives = commutator_primitives_from_scan(&puzzle, &scan_record, true);
            let dynamic_targets = commutator_targets(&puzzle, options.solver.target_mode);
            let unit_config = unit_closure_solver_config(
                &options.solver,
                options.commutator_suffix_time_limit_ms,
            );
            let unit_artifacts = prepare_solver(&puzzle, &unit_config)?;
            println!(
                "prepared {}-{} primitives={} unit_artifacts={} scan_elapsed={}ms",
                layout,
                difficulty,
                primitives.len(),
                artifact_summary(&unit_artifacts),
                scan_record.elapsed_ms
            );

            for &scramble_len in &options.scramble_lengths {
                for &iteration in &iterations {
                    let run_seed =
                        derive_seed(options.seed, layout, difficulty, scramble_len, iteration);
                    let scramble = generate_scramble(
                        &puzzle,
                        scramble_len,
                        run_seed,
                        options.scramble_profile,
                        options.avoid_same_tape,
                    );
                    let ariadne_solution_len = ariadne_solution_len(&puzzle, &scramble);
                    let mut scrambled_colors = puzzle.solved_colors.clone();
                    puzzle.apply_moves(&mut scrambled_colors, &scramble);
                    let static_target = vec![best_commutator_target(
                        &puzzle,
                        &scrambled_colors,
                        options.solver.target_mode,
                    )];
                    let greedy_targets = if options.commutator_dynamic_target {
                        dynamic_targets.as_slice()
                    } else {
                        static_target.as_slice()
                    };

                    let greedy = greedy_commutator_solve(
                        &puzzle,
                        &scrambled_colors,
                        greedy_targets,
                        &primitives,
                        options.commutator_greedy_steps,
                        options.commutator_plateau_lookahead,
                    );
                    let mut plateau_colors = scrambled_colors.clone();
                    puzzle.apply_moves(&mut plateau_colors, &greedy.moves);
                    let target = nearest_commutator_target(&plateau_colors, greedy_targets);
                    let mismatch_positions = mismatch_positions(&plateau_colors, target);
                    let plateau_mismatches = mismatch_positions.len();
                    let (exact_support_available, subset_support_count, contains_all_support_count) =
                        residual_support_stats(&mismatch_positions, &primitives);
                    let (target_min_mismatches, target_max_mismatches, target_min_count) =
                        target_mismatch_sensitivity(&plateau_colors, greedy_targets);

                    let direct = greedy_commutator_solve(
                        &puzzle,
                        &plateau_colors,
                        greedy_targets,
                        &primitives,
                        options.commutator_greedy_steps,
                        0,
                    );
                    let direct_opt = if options.solver.optimize {
                        optimize_solution(&puzzle, &plateau_colors, &direct.moves)
                    } else {
                        direct.moves.clone()
                    };
                    let mut direct_colors = plateau_colors.clone();
                    puzzle.apply_moves(&mut direct_colors, &direct_opt);
                    let direct_final_mismatches =
                        best_mismatches_to_targets(&direct_colors, greedy_targets);
                    let direct_found = is_target_solved(
                        &direct_colors,
                        &puzzle,
                        acceptance_target(options.solver.target_mode),
                    );
                    let mut direct_total_raw = greedy.moves.clone();
                    direct_total_raw.extend_from_slice(&direct_opt);
                    let direct_total_opt = if options.solver.optimize {
                        optimize_solution(&puzzle, &scrambled_colors, &direct_total_raw)
                    } else {
                        direct_total_raw
                    };

                    let unit_started = Instant::now();
                    let unit_result =
                        solve_puzzle(&puzzle, &plateau_colors, &unit_config, &unit_artifacts);
                    let mut unit_colors = plateau_colors.clone();
                    puzzle.apply_moves(&mut unit_colors, &unit_result.optimized_moves);
                    let unit_final_mismatches =
                        best_mismatches_to_targets(&unit_colors, &dynamic_targets);
                    let mut unit_total_raw = greedy.moves.clone();
                    unit_total_raw.extend_from_slice(&unit_result.optimized_moves);
                    let unit_total_opt = if options.solver.optimize {
                        optimize_solution(&puzzle, &scrambled_colors, &unit_total_raw)
                    } else {
                        unit_total_raw
                    };

                    let helper_started = Instant::now();
                    let helper = commutator_setup_tail_solve(
                        &puzzle,
                        &plateau_colors,
                        greedy_targets,
                        &primitives,
                        options.commutator_endgame_depth.max(1),
                        2,
                        &options,
                    );
                    let helper_opt = if options.solver.optimize {
                        optimize_solution(&puzzle, &plateau_colors, &helper.moves)
                    } else {
                        helper.moves.clone()
                    };
                    let mut helper_colors = plateau_colors.clone();
                    puzzle.apply_moves(&mut helper_colors, &helper_opt);
                    let helper_final_mismatches =
                        best_mismatches_to_targets(&helper_colors, greedy_targets);
                    let helper_found = helper.found
                        && is_target_solved(
                            &helper_colors,
                            &puzzle,
                            acceptance_target(options.solver.target_mode),
                        );
                    let mut helper_total_raw = greedy.moves.clone();
                    helper_total_raw.extend_from_slice(&helper_opt);
                    let helper_total_opt = if options.solver.optimize {
                        optimize_solution(&puzzle, &scrambled_colors, &helper_total_raw)
                    } else {
                        helper_total_raw
                    };

                    let helper_endgame_started = Instant::now();
                    let helper_endgame = if helper_found {
                        CommutatorEndgameSearchResult {
                            found: true,
                            reason: "already_solved_after_helper".to_string(),
                            moves: Vec::new(),
                            depth: 0,
                            nodes: 0,
                            best_mismatches: 0,
                        }
                    } else {
                        commutator_endgame_search(
                            &puzzle,
                            &helper_colors,
                            greedy_targets,
                            &primitives,
                            &options,
                        )
                    };
                    let helper_endgame_opt = if options.solver.optimize {
                        optimize_solution(&puzzle, &helper_colors, &helper_endgame.moves)
                    } else {
                        helper_endgame.moves.clone()
                    };
                    let mut helper_endgame_colors = helper_colors.clone();
                    puzzle.apply_moves(&mut helper_endgame_colors, &helper_endgame_opt);
                    let helper_endgame_final_mismatches =
                        best_mismatches_to_targets(&helper_endgame_colors, greedy_targets);
                    let helper_endgame_found = helper_endgame.found
                        && is_target_solved(
                            &helper_endgame_colors,
                            &puzzle,
                            acceptance_target(options.solver.target_mode),
                        );
                    let mut helper_endgame_total_raw = greedy.moves.clone();
                    helper_endgame_total_raw.extend_from_slice(&helper_opt);
                    helper_endgame_total_raw.extend_from_slice(&helper_endgame_opt);
                    let helper_endgame_total_opt = if options.solver.optimize {
                        optimize_solution(&puzzle, &scrambled_colors, &helper_endgame_total_raw)
                    } else {
                        helper_endgame_total_raw
                    };

                    let record = CommutatorDecompositionRecord {
                        layout,
                        difficulty,
                        target_mode: options.solver.target_mode,
                        scramble_len,
                        iteration: iteration + 1,
                        seed: run_seed,
                        ariadne_solution_len,
                        plateau_mismatches,
                        plateau_positions: join_usize(&mismatch_positions),
                        exact_support_available,
                        subset_support_count,
                        contains_all_support_count,
                        target_min_mismatches,
                        target_max_mismatches,
                        target_min_count,
                        direct_found,
                        direct_reason: direct.reason.clone(),
                        direct_steps: direct.steps,
                        direct_raw_len: direct.moves.len(),
                        direct_opt_len: direct_opt.len(),
                        direct_total_opt_len: direct_total_opt.len(),
                        direct_final_mismatches,
                        unit_found: unit_result.found,
                        unit_reason: unit_result.reason.clone(),
                        unit_raw_len: unit_result.raw_moves.len(),
                        unit_opt_len: unit_result.optimized_moves.len(),
                        unit_total_opt_len: unit_total_opt.len(),
                        unit_final_mismatches,
                        unit_nodes: unit_result.nodes,
                        unit_elapsed_ms: unit_started.elapsed().as_millis(),
                        helper_found,
                        helper_reason: helper.reason.clone(),
                        helper_steps: helper.steps,
                        helper_raw_len: helper.moves.len(),
                        helper_opt_len: helper_opt.len(),
                        helper_total_opt_len: helper_total_opt.len(),
                        helper_final_mismatches,
                        helper_nodes: helper.nodes,
                        helper_elapsed_ms: helper_started.elapsed().as_millis(),
                        helper_endgame_found,
                        helper_endgame_reason: helper_endgame.reason.clone(),
                        helper_endgame_raw_len: helper_endgame.moves.len(),
                        helper_endgame_opt_len: helper_endgame_opt.len(),
                        helper_endgame_total_opt_len: helper_endgame_total_opt.len(),
                        helper_endgame_final_mismatches,
                        helper_endgame_nodes: helper_endgame.nodes,
                        helper_endgame_elapsed_ms: helper_endgame_started.elapsed().as_millis(),
                    };

                    println!(
                        "{}-{} scramble={} iter={} plateau={} direct_found={} direct_tail={} direct_total={} helper_found={} helper_tail={} helper_total={} helper_time={}ms helper_endgame_found={} helper_endgame_tail={} helper_endgame_total={} helper_endgame_time={}ms unit_found={} unit_tail={} unit_total={} unit_time={}ms reason={}",
                        layout,
                        difficulty,
                        scramble_len,
                        iteration + 1,
                        record.plateau_mismatches,
                        record.direct_found,
                        record.direct_opt_len,
                        record.direct_total_opt_len,
                        record.helper_found,
                        record.helper_opt_len,
                        record.helper_total_opt_len,
                        record.helper_elapsed_ms,
                        record.helper_endgame_found,
                        record.helper_endgame_opt_len,
                        record.helper_endgame_total_opt_len,
                        record.helper_endgame_elapsed_ms,
                        record.unit_found,
                        record.unit_opt_len,
                        record.unit_total_opt_len,
                        record.unit_elapsed_ms,
                        record.unit_reason
                    );
                    records.push(record);
                }
            }
        }
    }

    write_commutator_decomposition_csv(&csv_path, &records)?;
    write_commutator_decomposition_markdown(&md_path, &options, &records)?;
    println!("CSV: {}", csv_path.display());
    println!("MD:  {}", md_path.display());
    Ok(())
}

fn run_ring_prefix_to_residue_audit(options: BenchOptions) -> io::Result<()> {
    fs::create_dir_all(&options.out_dir)?;

    let stamp = unix_stamp();
    let csv_path = options
        .out_dir
        .join(format!("ring_prefix_to_residue_audit_{stamp}.csv"));
    let md_path = options
        .out_dir
        .join(format!("ring_prefix_to_residue_audit_{stamp}.md"));
    let gates = [4usize, 6, 8, 10, 12];
    let mut records = Vec::new();

    println!(
        "ring-prefix-to-residue-audit target={} gates={:?} endgame_depth={} endgame_width={} endgame_time_limit_ms={} scramble_profile={} layouts={} difficulties={} scrambles={:?} iterations={} seed={}",
        options.solver.target_mode.label(),
        gates,
        options.commutator_endgame_depth,
        options.commutator_endgame_width,
        options.commutator_endgame_time_limit_ms,
        options.scramble_profile.label(),
        join_display(&options.layouts),
        join_display(&options.difficulties),
        options.scramble_lengths,
        iteration_label(&options),
        options.seed
    );

    let iterations = iteration_values(&options);
    for &layout in &options.layouts {
        for &difficulty in &options.difficulties {
            let puzzle = Puzzle::new(layout, difficulty);
            let artifacts = prepare_solver(&puzzle, &options.solver)?;
            let scan_record = scan_commutators(&puzzle, &options);
            let primitives = commutator_primitives_from_scan(&puzzle, &scan_record, true);
            let dynamic_targets = commutator_targets(&puzzle, options.solver.target_mode);
            println!(
                "prepared {}-{} artifacts={} primitives={} scan_elapsed={}ms",
                layout,
                difficulty,
                artifact_summary(&artifacts),
                primitives.len(),
                scan_record.elapsed_ms
            );

            for &scramble_len in &options.scramble_lengths {
                for &iteration in &iterations {
                    let run_seed =
                        derive_seed(options.seed, layout, difficulty, scramble_len, iteration);
                    let scramble = generate_scramble(
                        &puzzle,
                        scramble_len,
                        run_seed,
                        options.scramble_profile,
                        options.avoid_same_tape,
                    );
                    let ariadne_solution_len = ariadne_solution_len(&puzzle, &scramble);
                    let mut scrambled_colors = puzzle.solved_colors.clone();
                    puzzle.apply_moves(&mut scrambled_colors, &scramble);

                    for &gate in &gates {
                        let prefix = find_ring_residue_prefix(
                            &puzzle,
                            &scrambled_colors,
                            &options.solver,
                            &artifacts,
                            &dynamic_targets,
                            gate,
                        );

                        let mut tail_found = false;
                        let mut tail_reason = "prefix_not_found".to_string();
                        let mut tail_moves = Vec::new();
                        let mut tail_raw_len = 0usize;
                        let mut tail_opt_len = 0usize;
                        let mut tail_mismatches = prefix.mismatches;
                        let mut tail_nodes = 0u64;
                        let mut tail_elapsed_ms = 0u128;

                        if prefix.found {
                            let tail_started = Instant::now();
                            let tail = commutator_endgame_search(
                                &puzzle,
                                &prefix.colors,
                                &dynamic_targets,
                                &primitives,
                                &options,
                            );
                            tail_elapsed_ms = tail_started.elapsed().as_millis();
                            tail_found = tail.found;
                            tail_reason = tail.reason.clone();
                            tail_raw_len = tail.moves.len();
                            tail_nodes = tail.nodes;
                            let tail_opt = if options.solver.optimize {
                                optimize_solution(&puzzle, &prefix.colors, &tail.moves)
                            } else {
                                tail.moves.clone()
                            };
                            tail_opt_len = tail_opt.len();
                            let mut tail_colors = prefix.colors.clone();
                            puzzle.apply_moves(&mut tail_colors, &tail_opt);
                            tail_mismatches =
                                best_mismatches_to_targets(&tail_colors, &dynamic_targets);
                            tail_moves = tail_opt;
                        }

                        let mut total_raw = prefix.moves.clone();
                        total_raw.extend_from_slice(&tail_moves);
                        let total_opt = if options.solver.optimize {
                            optimize_solution(&puzzle, &scrambled_colors, &total_raw)
                        } else {
                            total_raw
                        };
                        let mut final_colors = scrambled_colors.clone();
                        puzzle.apply_moves(&mut final_colors, &total_opt);
                        let total_found = is_target_solved(
                            &final_colors,
                            &puzzle,
                            acceptance_target(options.solver.target_mode),
                        );
                        let final_mismatches =
                            best_mismatches_to_targets(&final_colors, &dynamic_targets);

                        let record = RingResidueAuditRecord {
                            layout,
                            difficulty,
                            target_mode: options.solver.target_mode,
                            scramble_len,
                            iteration: iteration + 1,
                            seed: run_seed,
                            ariadne_solution_len,
                            gate,
                            prefix_found: prefix.found,
                            prefix_reason: prefix.reason.clone(),
                            prefix_rank: prefix.rank_label.clone(),
                            prefix_profile: prefix.profile_label.clone(),
                            prefix_len: prefix.moves.len(),
                            prefix_mismatches: prefix.mismatches,
                            prefix_nodes: prefix.nodes,
                            prefix_elapsed_ms: prefix.elapsed_ms,
                            tail_found,
                            tail_reason,
                            tail_raw_len,
                            tail_opt_len,
                            tail_mismatches,
                            tail_nodes,
                            tail_elapsed_ms,
                            total_found,
                            total_opt_len: total_opt.len(),
                            final_mismatches,
                        };
                        println!(
                            "{}-{} scramble={} iter={} K={} prefix_found={} prefix_len={} prefix_mismatch={} prefix_time={}ms tail_found={} tail_opt={} total_found={} total_opt={} final_mismatch={}",
                            layout,
                            difficulty,
                            scramble_len,
                            iteration + 1,
                            gate,
                            record.prefix_found,
                            record.prefix_len,
                            record.prefix_mismatches,
                            record.prefix_elapsed_ms,
                            record.tail_found,
                            record.tail_opt_len,
                            record.total_found,
                            record.total_opt_len,
                            record.final_mismatches
                        );
                        records.push(record);
                    }
                }
            }
        }
    }

    write_ring_residue_csv(&csv_path, &records)?;
    write_ring_residue_markdown(&md_path, &options, &records)?;
    println!("CSV: {}", csv_path.display());
    println!("MD:  {}", md_path.display());
    Ok(())
}

fn run_last_k_repair_audit(options: BenchOptions) -> io::Result<()> {
    fs::create_dir_all(&options.out_dir)?;

    let stamp = unix_stamp();
    let csv_path = options
        .out_dir
        .join(format!("last_k_repair_audit_{stamp}.csv"));
    let md_path = options
        .out_dir
        .join(format!("last_k_repair_audit_{stamp}.md"));
    let gates = [2usize, 4, 6, 8, 10, 12];
    let mut records = Vec::new();

    println!(
        "last-k-repair-audit target={} gates={:?} table_depth={} scramble_profile={} layouts={} difficulties={} scrambles={:?} iterations={} seed={} max_len={} greedy_steps={}",
        options.solver.target_mode.label(),
        gates,
        options.commutator_endgame_depth,
        options.scramble_profile.label(),
        join_display(&options.layouts),
        join_display(&options.difficulties),
        options.scramble_lengths,
        iteration_label(&options),
        options.seed,
        options.commutator_max_len,
        options.commutator_greedy_steps
    );

    let iterations = iteration_values(&options);
    for &layout in &options.layouts {
        for &difficulty in &options.difficulties {
            let puzzle = Puzzle::new(layout, difficulty);
            let scan_record = scan_commutators(&puzzle, &options);
            let primitives = commutator_primitives_from_scan(&puzzle, &scan_record, true);
            let dynamic_targets = commutator_targets(&puzzle, options.solver.target_mode);
            let table = build_last_k_repair_table(
                &puzzle,
                &dynamic_targets,
                options.commutator_endgame_depth,
                &gates,
                options.solver.max_nodes,
                options.solver.time_limit_ms,
            );
            println!(
                "prepared {}-{} primitives={} table_states={} gate_counts={} depth_counts={} nodes={} elapsed={}ms truncated={} reason={} scan_elapsed={}ms",
                layout,
                difficulty,
                primitives.len(),
                table.repair.len(),
                gate_counts_text(&gates, &table.gate_counts),
                join_usize_counts(&table.depth_counts),
                table.nodes,
                table.elapsed_ms,
                table.truncated,
                table.reason,
                scan_record.elapsed_ms
            );

            for &scramble_len in &options.scramble_lengths {
                for &iteration in &iterations {
                    let run_seed =
                        derive_seed(options.seed, layout, difficulty, scramble_len, iteration);
                    let scramble = generate_scramble(
                        &puzzle,
                        scramble_len,
                        run_seed,
                        options.scramble_profile,
                        options.avoid_same_tape,
                    );
                    let ariadne_solution_len = ariadne_solution_len(&puzzle, &scramble);
                    let mut scrambled_colors = puzzle.solved_colors.clone();
                    puzzle.apply_moves(&mut scrambled_colors, &scramble);
                    let static_target = vec![best_commutator_target(
                        &puzzle,
                        &scrambled_colors,
                        options.solver.target_mode,
                    )];
                    let greedy_targets = if options.commutator_dynamic_target {
                        dynamic_targets.as_slice()
                    } else {
                        static_target.as_slice()
                    };

                    let greedy = greedy_commutator_solve(
                        &puzzle,
                        &scrambled_colors,
                        greedy_targets,
                        &primitives,
                        options.commutator_greedy_steps,
                        options.commutator_plateau_lookahead,
                    );
                    let greedy_prefix = if options.solver.optimize {
                        optimize_solution(&puzzle, &scrambled_colors, &greedy.moves)
                    } else {
                        greedy.moves.clone()
                    };
                    records.push(make_last_k_repair_record(
                        &puzzle,
                        layout,
                        difficulty,
                        options.solver.target_mode,
                        scramble_len,
                        iteration + 1,
                        run_seed,
                        ariadne_solution_len,
                        "plateau",
                        &scrambled_colors,
                        &greedy_prefix,
                        &dynamic_targets,
                        &table,
                        &gates,
                        options.commutator_endgame_depth,
                        format!("greedy:{}", greedy.reason),
                        options.solver.optimize,
                    ));

                    let mut plateau_colors = scrambled_colors.clone();
                    puzzle.apply_moves(&mut plateau_colors, &greedy_prefix);
                    let helper = commutator_setup_tail_solve(
                        &puzzle,
                        &plateau_colors,
                        greedy_targets,
                        &primitives,
                        options.commutator_endgame_depth.max(1),
                        2,
                        &options,
                    );
                    let helper_opt = if options.solver.optimize {
                        optimize_solution(&puzzle, &plateau_colors, &helper.moves)
                    } else {
                        helper.moves.clone()
                    };
                    let mut helper_prefix_raw = greedy_prefix.clone();
                    helper_prefix_raw.extend_from_slice(&helper_opt);
                    let helper_prefix = if options.solver.optimize {
                        optimize_solution(&puzzle, &scrambled_colors, &helper_prefix_raw)
                    } else {
                        helper_prefix_raw
                    };
                    records.push(make_last_k_repair_record(
                        &puzzle,
                        layout,
                        difficulty,
                        options.solver.target_mode,
                        scramble_len,
                        iteration + 1,
                        run_seed,
                        ariadne_solution_len,
                        "helper",
                        &scrambled_colors,
                        &helper_prefix,
                        &dynamic_targets,
                        &table,
                        &gates,
                        options.commutator_endgame_depth,
                        format!("helper:{}", helper.reason),
                        options.solver.optimize,
                    ));

                    if let Some(last) = records.last() {
                        println!(
                            "{}-{} scramble={} iter={} source={} mismatches={} hit={} suffix={} total_found={} total_opt={} final_mismatch={}",
                            layout,
                            difficulty,
                            scramble_len,
                            iteration + 1,
                            last.source,
                            last.source_mismatches,
                            last.hit,
                            last.hit_suffix_len,
                            last.total_found,
                            last.total_opt_len,
                            last.final_mismatches
                        );
                    }
                }
            }
        }
    }

    write_last_k_repair_csv(&csv_path, &records)?;
    write_last_k_repair_markdown(&md_path, &options, &records)?;
    println!("CSV: {}", csv_path.display());
    println!("MD:  {}", md_path.display());
    Ok(())
}

fn build_last_k_repair_table(
    puzzle: &Puzzle,
    targets: &[Vec<Color>],
    max_depth: usize,
    gates: &[usize],
    max_nodes: u64,
    time_limit_ms: u64,
) -> LastKRepairTable {
    let started = Instant::now();
    let max_gate = gates.iter().copied().max().unwrap_or(0);
    let mut limits = Limits::new(max_nodes, time_limit_ms);
    let mut nodes = 0u64;
    let mut seen: HashMap<Key, PathBits> = HashMap::new();
    let mut repair: HashMap<Key, PathBits> = HashMap::new();
    let mut frontier = Vec::new();
    let mut gate_counts = vec![0usize; gates.len()];
    let mut depth_counts = vec![0usize; max_depth + 1];
    let mut truncated = false;
    let mut reason = "complete".to_string();

    for seed in targets {
        let key = state_key(seed);
        if seen.insert(key, PathBits::default()).is_none() {
            frontier.push(TableEntry {
                colors: seed.clone(),
                path: PathBits::default(),
                last_move: None,
            });
            depth_counts[0] += 1;
            record_last_k_repair_candidate(
                puzzle,
                seed,
                targets,
                key,
                PathBits::default(),
                max_gate,
                gates,
                &mut repair,
                &mut gate_counts,
            );
        }
    }

    'depths: for depth in 0..max_depth {
        let mut next = Vec::new();
        for entry in frontier {
            for move_index in 0..puzzle.moves.len() {
                if is_immediate_inverse(puzzle, entry.last_move, move_index) {
                    continue;
                }
                let mut colors = entry.colors.clone();
                puzzle.apply_move(&mut colors, move_index);
                nodes += 1;
                if limits.exceeded(nodes) {
                    truncated = true;
                    reason = limits
                        .stop_reason
                        .clone()
                        .unwrap_or_else(|| "stopped".to_string());
                    break 'depths;
                }

                let key = state_key(&colors);
                if seen.contains_key(&key) {
                    continue;
                }
                let inverse = puzzle.inverse_index(move_index);
                let path = entry.path.prepend(inverse);
                seen.insert(key, path);
                depth_counts[depth + 1] += 1;
                record_last_k_repair_candidate(
                    puzzle,
                    &colors,
                    targets,
                    key,
                    path,
                    max_gate,
                    gates,
                    &mut repair,
                    &mut gate_counts,
                );
                next.push(TableEntry {
                    colors,
                    path,
                    last_move: Some(move_index),
                });
            }
        }
        frontier = next;
        if frontier.is_empty() {
            break;
        }
    }

    LastKRepairTable {
        repair,
        gate_counts,
        depth_counts,
        nodes,
        elapsed_ms: started.elapsed().as_millis(),
        truncated,
        reason,
    }
}

fn record_last_k_repair_candidate(
    _puzzle: &Puzzle,
    colors: &[Color],
    targets: &[Vec<Color>],
    key: Key,
    path: PathBits,
    max_gate: usize,
    gates: &[usize],
    repair: &mut HashMap<Key, PathBits>,
    gate_counts: &mut [usize],
) {
    let mismatches = best_mismatches_to_targets(colors, targets);
    if mismatches > max_gate {
        return;
    }
    if repair.insert(key, path).is_none() {
        for (index, gate) in gates.iter().copied().enumerate() {
            if mismatches <= gate {
                gate_counts[index] += 1;
            }
        }
    }
}

fn make_last_k_repair_record(
    puzzle: &Puzzle,
    layout: LayoutId,
    difficulty: Difficulty,
    target_mode: TargetMode,
    scramble_len: usize,
    iteration: usize,
    seed: u64,
    ariadne_solution_len: usize,
    source: &str,
    scrambled_colors: &[Color],
    source_prefix: &[MoveIndex],
    targets: &[Vec<Color>],
    table: &LastKRepairTable,
    gates: &[usize],
    table_depth: usize,
    source_reason: String,
    optimize: bool,
) -> LastKRepairRecord {
    let mut source_colors = scrambled_colors.to_vec();
    puzzle.apply_moves(&mut source_colors, source_prefix);
    let source_mismatches = best_mismatches_to_targets(&source_colors, targets);
    let hit_path = table.repair.get(&state_key(&source_colors)).copied();
    let hit = hit_path.is_some();
    let hit_suffix = hit_path.map(PathBits::to_vec).unwrap_or_default();
    let hit_suffix_len = hit_suffix.len();
    let mut total_raw = source_prefix.to_vec();
    total_raw.extend_from_slice(&hit_suffix);
    let total_opt = if optimize {
        optimize_solution(puzzle, scrambled_colors, &total_raw)
    } else {
        total_raw
    };
    let mut final_colors = scrambled_colors.to_vec();
    puzzle.apply_moves(&mut final_colors, &total_opt);
    let total_found = is_target_solved(&final_colors, puzzle, acceptance_target(target_mode));
    let final_mismatches = best_mismatches_to_targets(&final_colors, targets);

    LastKRepairRecord {
        layout,
        difficulty,
        target_mode,
        scramble_len,
        iteration,
        seed,
        ariadne_solution_len,
        source: source.to_string(),
        table_depth,
        table_states: table.repair.len(),
        table_gate_counts: gate_counts_text(gates, &table.gate_counts),
        table_depth_counts: join_usize_counts(&table.depth_counts),
        table_nodes: table.nodes,
        table_elapsed_ms: table.elapsed_ms,
        table_truncated: table.truncated,
        table_reason: table.reason.clone(),
        source_mismatches,
        source_prefix_len: source_prefix.len(),
        source_reason,
        hit,
        hit_suffix_len,
        total_found,
        total_opt_len: total_opt.len(),
        final_mismatches,
    }
}

fn gate_counts_text(gates: &[usize], counts: &[usize]) -> String {
    gates
        .iter()
        .zip(counts.iter())
        .map(|(gate, count)| format!("{gate}:{count}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn write_last_k_repair_csv(path: &PathBuf, records: &[LastKRepairRecord]) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(
        file,
        "layout,difficulty,target,scramble_len,iteration,seed,ariadne_solution_len,source,table_depth,table_states,table_gate_counts,table_depth_counts,table_nodes,table_elapsed_ms,table_truncated,table_reason,source_mismatches,source_prefix_len,source_reason,hit,hit_suffix_len,total_found,total_opt_len,final_mismatches"
    )?;
    for r in records {
        writeln!(
            file,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            r.layout,
            r.difficulty,
            r.target_mode.label(),
            r.scramble_len,
            r.iteration,
            r.seed,
            r.ariadne_solution_len,
            csv(&r.source),
            r.table_depth,
            r.table_states,
            csv(&r.table_gate_counts),
            csv(&r.table_depth_counts),
            r.table_nodes,
            r.table_elapsed_ms,
            r.table_truncated,
            csv(&r.table_reason),
            r.source_mismatches,
            r.source_prefix_len,
            csv(&r.source_reason),
            r.hit,
            r.hit_suffix_len,
            r.total_found,
            r.total_opt_len,
            r.final_mismatches
        )?;
    }
    Ok(())
}

fn write_last_k_repair_markdown(
    path: &PathBuf,
    options: &BenchOptions,
    records: &[LastKRepairRecord],
) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(file, "# SkimmIQ Last-K Repair Audit")?;
    writeln!(file)?;
    writeln!(file, "- target: `{}`", options.solver.target_mode.label())?;
    writeln!(file, "- layouts: `{}`", join_display(&options.layouts))?;
    writeln!(
        file,
        "- difficulties: `{}`",
        join_display(&options.difficulties)
    )?;
    writeln!(file, "- scrambles: `{:?}`", options.scramble_lengths)?;
    writeln!(file, "- iterations: `{}`", iteration_label(options))?;
    writeln!(file, "- seed: `{}`", options.seed)?;
    writeln!(
        file,
        "- table_depth: `{}` (`--commutator-endgame-depth`)",
        options.commutator_endgame_depth
    )?;
    writeln!(file)?;
    writeln!(
        file,
        "This audit builds an offline reverse table from solved Android targets, but only stores states whose color mismatch count is within gates `2,4,6,8,10,12`. It then tests whether commutator plateau/helper residue states hit that shippable subset. The build still needs the full BFS frontier; the reported `table_states` is the filtered table size that would be shipped."
    )?;
    writeln!(file)?;

    let mut groups: BTreeMap<(LayoutId, Difficulty, usize, String), Vec<&LastKRepairRecord>> =
        BTreeMap::new();
    for record in records {
        groups
            .entry((
                record.layout,
                record.difficulty,
                record.scramble_len,
                record.source.clone(),
            ))
            .or_default()
            .push(record);
    }

    writeln!(
        file,
        "| Layout | Difficulty | Scramble | Source | Samples | Hits | Found | Mean Source Mismatch | Mean Prefix | Mean Suffix | Mean Total Opt | Mean Final Mismatch | Table States | Gate Counts | Depth Counts | Table Time ms | Table Nodes | Table Reason |"
    )?;
    writeln!(
        file,
        "|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|---|---:|---:|---|"
    )?;
    for ((layout, difficulty, scramble_len, source), group) in &groups {
        let hits = group.iter().filter(|record| record.hit).count();
        let found = group.iter().filter(|record| record.total_found).count();
        let mismatches = group
            .iter()
            .map(|record| record.source_mismatches)
            .collect::<Vec<_>>();
        let prefixes = group
            .iter()
            .map(|record| record.source_prefix_len)
            .collect::<Vec<_>>();
        let suffixes = group
            .iter()
            .map(|record| record.hit_suffix_len)
            .collect::<Vec<_>>();
        let totals = group
            .iter()
            .map(|record| record.total_opt_len)
            .collect::<Vec<_>>();
        let finals = group
            .iter()
            .map(|record| record.final_mismatches)
            .collect::<Vec<_>>();
        let first = group[0];
        writeln!(
            file,
            "| {} | {} | {} | `{}` | {} | {}/{} | {}/{} | {} | {} | {} | {} | {} | {} | `{}` | `{}` | {} | {} | `{}` |",
            layout,
            difficulty,
            scramble_len,
            source,
            group.len(),
            hits,
            group.len(),
            found,
            group.len(),
            fmt_opt_f64(mean_usize(&mismatches)),
            fmt_opt_f64(mean_usize(&prefixes)),
            fmt_opt_f64(mean_usize(&suffixes)),
            fmt_opt_f64(mean_usize(&totals)),
            fmt_opt_f64(mean_usize(&finals)),
            first.table_states,
            first.table_gate_counts,
            first.table_depth_counts,
            first.table_elapsed_ms,
            first.table_nodes,
            first.table_reason
        )?;
    }

    writeln!(file)?;
    writeln!(file, "## Iteration Details")?;
    writeln!(file)?;
    writeln!(
        file,
        "| Iter | Source | Seed | Ariadne | Mismatches | Prefix | Hit | Suffix | Found | Total Opt | Final Mismatch | Reason |"
    )?;
    writeln!(
        file,
        "|---:|---|---:|---:|---:|---:|---|---:|---|---:|---:|---|"
    )?;
    for record in records {
        writeln!(
            file,
            "| {} | `{}` | {} | {} | {} | {} | {} | {} | {} | {} | {} | `{}` |",
            record.iteration,
            record.source,
            record.seed,
            record.ariadne_solution_len,
            record.source_mismatches,
            record.source_prefix_len,
            record.hit,
            record.hit_suffix_len,
            record.total_found,
            record.total_opt_len,
            record.final_mismatches,
            record.source_reason
        )?;
    }

    Ok(())
}

fn run_commutator_branch_audit(options: BenchOptions) -> io::Result<()> {
    fs::create_dir_all(&options.out_dir)?;

    let stamp = unix_stamp();
    let csv_path = options
        .out_dir
        .join(format!("commutator_branch_audit_{stamp}.csv"));
    let md_path = options
        .out_dir
        .join(format!("commutator_branch_audit_{stamp}.md"));
    let mut records = Vec::new();
    let gates = [2usize, 4, 6, 8, 10, 12];

    println!(
        "commutator-branch-audit target={} gates={:?} branch_cap={} scramble_profile={} layouts={} difficulties={} scrambles={:?} iterations={} seed={} max_len={}",
        options.solver.target_mode.label(),
        gates,
        options.commutator_top,
        options.scramble_profile.label(),
        join_display(&options.layouts),
        join_display(&options.difficulties),
        options.scramble_lengths,
        iteration_label(&options),
        options.seed,
        options.commutator_max_len
    );

    let iterations = iteration_values(&options);
    for &layout in &options.layouts {
        for &difficulty in &options.difficulties {
            let puzzle = Puzzle::new(layout, difficulty);
            let scan_record = scan_commutators(&puzzle, &options);
            let primitives = commutator_primitives_from_scan(&puzzle, &scan_record, true);
            let dynamic_targets = commutator_targets(&puzzle, options.solver.target_mode);
            println!(
                "prepared {}-{} primitives={} scan_elapsed={}ms",
                layout,
                difficulty,
                primitives.len(),
                scan_record.elapsed_ms
            );

            for &scramble_len in &options.scramble_lengths {
                for &iteration in &iterations {
                    let run_seed =
                        derive_seed(options.seed, layout, difficulty, scramble_len, iteration);
                    let scramble = generate_scramble(
                        &puzzle,
                        scramble_len,
                        run_seed,
                        options.scramble_profile,
                        options.avoid_same_tape,
                    );
                    let ariadne_moves = ariadne_solution_moves(&puzzle, &scramble);
                    let mut colors = puzzle.solved_colors.clone();
                    puzzle.apply_moves(&mut colors, &scramble);
                    let mut recorded = HashSet::new();

                    for step in 0..=ariadne_moves.len() {
                        let target = nearest_commutator_target(&colors, &dynamic_targets);
                        let mismatches = color_mismatches(&colors, target);
                        for &gate in &gates {
                            if mismatches <= gate && recorded.insert(gate) {
                                records.push(make_commutator_branch_record(
                                    &puzzle,
                                    &colors,
                                    target,
                                    &primitives,
                                    &options,
                                    layout,
                                    difficulty,
                                    scramble_len,
                                    iteration + 1,
                                    run_seed,
                                    format!("ariadne<={gate}"),
                                    gate,
                                    step,
                                    ariadne_moves.len().saturating_sub(step),
                                ));
                            }
                        }
                        if step == ariadne_moves.len() {
                            break;
                        }
                        puzzle.apply_move(&mut colors, ariadne_moves[step]);
                    }

                    let mut scrambled_colors = puzzle.solved_colors.clone();
                    puzzle.apply_moves(&mut scrambled_colors, &scramble);
                    let static_target = vec![best_commutator_target(
                        &puzzle,
                        &scrambled_colors,
                        options.solver.target_mode,
                    )];
                    let greedy_targets = if options.commutator_dynamic_target {
                        dynamic_targets.as_slice()
                    } else {
                        static_target.as_slice()
                    };
                    let greedy = greedy_commutator_solve(
                        &puzzle,
                        &scrambled_colors,
                        greedy_targets,
                        &primitives,
                        options.commutator_greedy_steps,
                        options.commutator_plateau_lookahead,
                    );
                    let mut plateau_colors = scrambled_colors.clone();
                    puzzle.apply_moves(&mut plateau_colors, &greedy.moves);
                    let plateau_target = nearest_commutator_target(&plateau_colors, greedy_targets);
                    let plateau_mismatches = color_mismatches(&plateau_colors, plateau_target);
                    records.push(make_commutator_branch_record(
                        &puzzle,
                        &plateau_colors,
                        plateau_target,
                        &primitives,
                        &options,
                        layout,
                        difficulty,
                        scramble_len,
                        iteration + 1,
                        run_seed,
                        "greedy-plateau".to_string(),
                        plateau_mismatches,
                        greedy.steps,
                        0,
                    ));

                    println!(
                        "{}-{} scramble={} iter={} ariadne_len={} gates_recorded={} plateau_mismatches={}",
                        layout,
                        difficulty,
                        scramble_len,
                        iteration + 1,
                        ariadne_moves.len(),
                        recorded.len(),
                        plateau_mismatches
                    );
                }
            }
        }
    }

    write_commutator_branch_csv(&csv_path, &records)?;
    write_commutator_branch_markdown(&md_path, &options, &records)?;
    println!("CSV: {}", csv_path.display());
    println!("MD:  {}", md_path.display());
    Ok(())
}

fn run_beam_direction_survival_audit(options: BenchOptions) -> io::Result<()> {
    fs::create_dir_all(&options.out_dir)?;

    let stamp = unix_stamp();
    let csv_path = options
        .out_dir
        .join(format!("beam_direction_survival_audit_{stamp}.csv"));
    let md_path = options
        .out_dir
        .join(format!("beam_direction_survival_audit_{stamp}.md"));
    let mut records = Vec::new();
    let iterations = iteration_values(&options);
    let tier = options.solver.tiers.first().copied().unwrap_or(Tier {
        max_depth: DEFAULT_DIRECTION_SURVIVAL_DEPTH,
        width: 700,
        restarts: 1,
    });
    let audit_depth = tier.max_depth.min(DEFAULT_DIRECTION_SURVIVAL_DEPTH);

    println!(
        "beam-direction-survival-audit target={} operation_profile={} scramble_profile={} layouts={} difficulties={} scrambles={:?} iterations={} seed={} depth={} width={} pattern_db={}",
        options.solver.target_mode.label(),
        options.solver.operation_profile.label(),
        options.scramble_profile.label(),
        join_display(&options.layouts),
        join_display(&options.difficulties),
        options.scramble_lengths,
        iterations.len(),
        options.seed,
        audit_depth,
        tier.width,
        options.solver.pattern_db_enabled
    );

    for &layout in &options.layouts {
        for &difficulty in &options.difficulties {
            let puzzle = Puzzle::new(layout, difficulty);
            let operation_profile = options.solver.operation_profile.for_layout(layout);
            let operations = build_operations(&puzzle, operation_profile);

            for target_mode in target_variants(options.solver.target_mode) {
                let mut solver = options.solver.clone();
                solver.target_mode = target_mode;
                let mut pattern_nodes = 0u64;
                let mut pattern_limits = Limits::new(solver.max_nodes, solver.time_limit_ms);
                let pattern_db =
                    build_pattern_db(&puzzle, &solver, &mut pattern_nodes, &mut pattern_limits);
                println!(
                    "prepared {}-{} target={} operations={} profile={} pattern_db={} pattern_nodes={}",
                    layout,
                    difficulty,
                    target_mode.label(),
                    operations.len(),
                    operation_profile.label(),
                    pattern_db.is_some(),
                    pattern_nodes
                );

                for &scramble_len in &options.scramble_lengths {
                    for &iteration in &iterations {
                        let seed =
                            derive_seed(options.seed, layout, difficulty, scramble_len, iteration);
                        let scramble = generate_scramble(
                            &puzzle,
                            scramble_len,
                            seed,
                            options.scramble_profile,
                            options.avoid_same_tape,
                        );
                        let ariadne_solution = ariadne_solution_moves(&puzzle, &scramble);
                        let Some(&ariadne_first_move) = ariadne_solution.first() else {
                            continue;
                        };
                        let mut colors = puzzle.solved_colors.clone();
                        puzzle.apply_moves(&mut colors, &scramble);
                        let mut run_records = evaluate_beam_direction_survival(
                            &puzzle,
                            &solver,
                            operation_profile,
                            &operations,
                            pattern_db.as_ref(),
                            scramble_len,
                            iteration,
                            seed,
                            ariadne_solution.len(),
                            ariadne_first_move,
                            tier.width,
                            audit_depth,
                            &colors,
                        );
                        let extinction = run_records
                            .iter()
                            .find_map(|record| record.extinction_layer);
                        println!(
                            "{}-{} target={} scramble={} iter={} ariadne={} first={} extinction={} final_alive={} final_dirs={}",
                            layout,
                            difficulty,
                            target_mode.label(),
                            scramble_len,
                            iteration + 1,
                            ariadne_solution.len(),
                            puzzle.move_text(ariadne_first_move),
                            fmt_opt_usize(extinction),
                            run_records.last().is_some_and(|record| record.survival),
                            run_records
                                .last()
                                .map(|record| record.direction_count)
                                .unwrap_or_default()
                        );
                        records.append(&mut run_records);
                    }
                }
            }
        }
    }

    write_beam_direction_survival_csv(&csv_path, &records)?;
    write_beam_direction_survival_markdown(&md_path, &options, &records)?;
    println!("CSV: {}", csv_path.display());
    println!("MD:  {}", md_path.display());
    Ok(())
}

fn run_beam_prefix_survival_audit(options: BenchOptions) -> io::Result<()> {
    fs::create_dir_all(&options.out_dir)?;

    let stamp = unix_stamp();
    let csv_path = options
        .out_dir
        .join(format!("beam_prefix_survival_audit_{stamp}.csv"));
    let md_path = options
        .out_dir
        .join(format!("beam_prefix_survival_audit_{stamp}.md"));
    let mut records = Vec::new();
    let iterations = iteration_values(&options);
    let tier = options.solver.tiers.first().copied().unwrap_or(Tier {
        max_depth: DEFAULT_DIRECTION_SURVIVAL_DEPTH,
        width: 700,
        restarts: 1,
    });
    let audit_depth = tier.max_depth.min(DEFAULT_DIRECTION_SURVIVAL_DEPTH);

    println!(
        "beam-prefix-survival-audit target={} operation_profile={} scramble_profile={} layouts={} difficulties={} scrambles={:?} iterations={} seed={} depth={} width={} pattern_db={}",
        options.solver.target_mode.label(),
        options.solver.operation_profile.label(),
        options.scramble_profile.label(),
        join_display(&options.layouts),
        join_display(&options.difficulties),
        options.scramble_lengths,
        iterations.len(),
        options.seed,
        audit_depth,
        tier.width,
        options.solver.pattern_db_enabled
    );

    for &layout in &options.layouts {
        for &difficulty in &options.difficulties {
            let puzzle = Puzzle::new(layout, difficulty);
            let operation_profile = options.solver.operation_profile.for_layout(layout);
            let operations = build_operations(&puzzle, operation_profile);

            for target_mode in target_variants(options.solver.target_mode) {
                let mut solver = options.solver.clone();
                solver.target_mode = target_mode;
                let mut pattern_nodes = 0u64;
                let mut pattern_limits = Limits::new(solver.max_nodes, solver.time_limit_ms);
                let pattern_db =
                    build_pattern_db(&puzzle, &solver, &mut pattern_nodes, &mut pattern_limits);
                println!(
                    "prepared {}-{} target={} operations={} profile={} pattern_db={} pattern_nodes={}",
                    layout,
                    difficulty,
                    target_mode.label(),
                    operations.len(),
                    operation_profile.label(),
                    pattern_db.is_some(),
                    pattern_nodes
                );

                for &scramble_len in &options.scramble_lengths {
                    for &iteration in &iterations {
                        let seed =
                            derive_seed(options.seed, layout, difficulty, scramble_len, iteration);
                        let scramble = generate_scramble(
                            &puzzle,
                            scramble_len,
                            seed,
                            options.scramble_profile,
                            options.avoid_same_tape,
                        );
                        let ariadne_solution = ariadne_solution_moves(&puzzle, &scramble);
                        if ariadne_solution.is_empty() {
                            continue;
                        }
                        let mut colors = puzzle.solved_colors.clone();
                        puzzle.apply_moves(&mut colors, &scramble);
                        let mut run_records = evaluate_beam_prefix_survival(
                            &puzzle,
                            &solver,
                            operation_profile,
                            &operations,
                            pattern_db.as_ref(),
                            scramble_len,
                            iteration,
                            seed,
                            &ariadne_solution,
                            tier.width,
                            audit_depth,
                            &colors,
                        );
                        println!(
                            "{}-{} target={} scramble={} iter={} ariadne={} max_prefix_final={} prefix{}_alive={} state{}_alive={}",
                            layout,
                            difficulty,
                            target_mode.label(),
                            scramble_len,
                            iteration + 1,
                            ariadne_solution.len(),
                            run_records
                                .last()
                                .map(|record| record.max_matching_prefix)
                                .unwrap_or_default(),
                            audit_depth,
                            run_records
                                .last()
                                .is_some_and(|record| record.path_prefix_alive),
                            audit_depth,
                            run_records
                                .last()
                                .is_some_and(|record| record.prefix_state_alive)
                        );
                        records.append(&mut run_records);
                    }
                }
            }
        }
    }

    write_beam_prefix_survival_csv(&csv_path, &records)?;
    write_beam_prefix_survival_markdown(&md_path, &options, &records)?;
    println!("CSV: {}", csv_path.display());
    println!("MD:  {}", md_path.display());
    Ok(())
}

fn run_backward_midpoint_audit(options: BenchOptions) -> io::Result<()> {
    fs::create_dir_all(&options.out_dir)?;

    let stamp = unix_stamp();
    let csv_path = options
        .out_dir
        .join(format!("backward_midpoint_audit_{stamp}.csv"));
    let md_path = options
        .out_dir
        .join(format!("backward_midpoint_audit_{stamp}.md"));
    let mut records = Vec::new();
    let iterations = iteration_values(&options);
    let tier = options.solver.tiers.first().copied().unwrap_or(Tier {
        max_depth: 10,
        width: 3000,
        restarts: 1,
    });
    let audit_depth = tier.max_depth;
    let midpoint_step = audit_depth;

    println!(
        "backward-midpoint-audit target={} operation_profile={} scramble_profile={} layouts={} difficulties={} scrambles={:?} iterations={} seed={} depth={} width={} midpoint_step={} pattern_db={}",
        options.solver.target_mode.label(),
        options.solver.operation_profile.label(),
        options.scramble_profile.label(),
        join_display(&options.layouts),
        join_display(&options.difficulties),
        options.scramble_lengths,
        iterations.len(),
        options.seed,
        audit_depth,
        tier.width,
        midpoint_step,
        options.solver.pattern_db_enabled
    );

    for &layout in &options.layouts {
        for &difficulty in &options.difficulties {
            let puzzle = Puzzle::new(layout, difficulty);
            let operation_profile = options.solver.operation_profile.for_layout(layout);
            let forward_operations = build_operations(&puzzle, operation_profile);
            let backward_operations = invert_operations(&puzzle, &forward_operations);

            for target_mode in target_variants(options.solver.target_mode) {
                let mut solver = options.solver.clone();
                solver.target_mode = target_mode;
                let mut pattern_nodes = 0u64;
                let mut pattern_limits = Limits::new(solver.max_nodes, solver.time_limit_ms);
                let pattern_db =
                    build_pattern_db(&puzzle, &solver, &mut pattern_nodes, &mut pattern_limits);
                let target_states = reverse_table_seeds(&puzzle, target_mode);
                println!(
                    "prepared {}-{} target={} forward_ops={} backward_ops={} profile={} target_states={} pattern_db={} pattern_nodes={}",
                    layout,
                    difficulty,
                    target_mode.label(),
                    forward_operations.len(),
                    backward_operations.len(),
                    operation_profile.label(),
                    target_states.len(),
                    pattern_db.is_some(),
                    pattern_nodes
                );

                for &scramble_len in &options.scramble_lengths {
                    for &iteration in &iterations {
                        let seed =
                            derive_seed(options.seed, layout, difficulty, scramble_len, iteration);
                        let scramble = generate_scramble(
                            &puzzle,
                            scramble_len,
                            seed,
                            options.scramble_profile,
                            options.avoid_same_tape,
                        );
                        let ariadne_solution = ariadne_solution_moves(&puzzle, &scramble);
                        if ariadne_solution.is_empty() {
                            continue;
                        }
                        let mut scrambled_colors = puzzle.solved_colors.clone();
                        puzzle.apply_moves(&mut scrambled_colors, &scramble);
                        let record = evaluate_backward_midpoint(
                            &puzzle,
                            &solver,
                            operation_profile,
                            &backward_operations,
                            pattern_db.as_ref(),
                            scramble_len,
                            iteration,
                            seed,
                            &ariadne_solution,
                            &scrambled_colors,
                            &target_states,
                            tier.width,
                            audit_depth,
                            midpoint_step,
                        );
                        println!(
                            "{}-{} target={} scramble={} iter={} ariadne={} step={} candidate_hit={} selected_hit={} final_hit={} best_rank={}",
                            layout,
                            difficulty,
                            target_mode.label(),
                            scramble_len,
                            iteration + 1,
                            ariadne_solution.len(),
                            record.midpoint_step,
                            record.candidate_hit,
                            record.selected_hit,
                            record.final_frontier_hit,
                            fmt_opt_usize(record.best_candidate_rank)
                        );
                        records.push(record);
                    }
                }
            }
        }
    }

    write_backward_midpoint_audit_csv(&csv_path, &records)?;
    write_backward_midpoint_audit_markdown(&md_path, &options, &records)?;
    println!("CSV: {}", csv_path.display());
    println!("MD:  {}", md_path.display());
    Ok(())
}

fn build_multi_start_pdb(
    puzzle: &Puzzle,
    options: &BenchOptions,
    target_mode: TargetMode,
    pdb_depth: usize,
) -> AxisRingPdb {
    let started = Instant::now();
    let seed_set = collect_pdb_seed_states(puzzle, options, target_mode);

    let mut distances: HashMap<Key, u8> = HashMap::new();
    let mut frontier = Vec::new();
    let mut depth_counts = vec![0usize; pdb_depth + 1];
    let mut nodes = seed_set.nodes;
    let mut truncated = seed_set.truncated;
    let mut reason = seed_set.reason;
    let mut limits = Limits::new(options.solver.max_nodes, options.solver.time_limit_ms);

    for colors in seed_set.states {
        let key = state_key(&colors);
        if distances.contains_key(&key) {
            continue;
        }
        distances.insert(key, 0);
        depth_counts[0] += 1;
        frontier.push(AxisRingPdbEntry {
            colors,
            last_move: None,
        });
    }

    let target_states = depth_counts[0];
    if !truncated {
        'depth_loop: for current_depth in 0..pdb_depth {
            let mut next = Vec::new();
            let next_distance = (current_depth + 1).min(u8::MAX as usize) as u8;
            let keep_next_frontier = current_depth + 1 < pdb_depth;
            for entry in frontier {
                for move_index in 0..puzzle.moves.len() {
                    if is_immediate_inverse(puzzle, entry.last_move, move_index) {
                        continue;
                    }

                    let mut colors = entry.colors.clone();
                    puzzle.apply_move(&mut colors, move_index);
                    nodes += 1;
                    if limits.exceeded(nodes) {
                        truncated = true;
                        reason = limits
                            .stop_reason
                            .clone()
                            .unwrap_or_else(|| "axis-ring-pdb expansion stopped".to_string());
                        break 'depth_loop;
                    }

                    let key = state_key(&colors);
                    if distances.contains_key(&key) {
                        continue;
                    }
                    distances.insert(key, next_distance);
                    depth_counts[current_depth + 1] += 1;
                    if keep_next_frontier {
                        next.push(AxisRingPdbEntry {
                            colors,
                            last_move: Some(move_index),
                        });
                    }
                }
            }
            frontier = next;
            if frontier.is_empty() {
                break;
            }
        }
    }

    AxisRingPdb {
        distances,
        depth_counts,
        target_states,
        max_axis_suffix: seed_set.max_suffix,
        nodes,
        elapsed_ms: started.elapsed().as_millis(),
        truncated,
        reason,
    }
}

fn collect_pdb_seed_states(
    puzzle: &Puzzle,
    options: &BenchOptions,
    target_mode: TargetMode,
) -> PdbSeedSet {
    match options.pdb_seed_source {
        PdbSeedSource::AxisRing => collect_axis_ring_seed_states(puzzle, options, target_mode),
        PdbSeedSource::AriadneMidpoints => collect_ariadne_midpoint_seed_states(puzzle, options),
        PdbSeedSource::RandomWalk => collect_random_walk_seed_states(puzzle, options),
    }
}

fn collect_axis_ring_seed_states(
    puzzle: &Puzzle,
    options: &BenchOptions,
    target_mode: TargetMode,
) -> PdbSeedSet {
    let mut config = options.solver.clone();
    config.target_expand_depth = options.axis_ring_rescue_expand_depth;
    let mut states = Vec::new();
    let mut nodes = 0u64;
    let mut max_suffix = 0usize;
    let mut truncated = false;
    let mut reason = "complete".to_string();
    let mut limits = Limits::new(options.solver.max_nodes, options.solver.time_limit_ms);

    'axis_loop: for axis in 0..3_u8 {
        let moves = axis_moves(puzzle, axis);
        if moves.is_empty() {
            continue;
        }
        let axis_artifacts = match build_restricted_target_artifacts(
            puzzle,
            target_mode,
            moves,
            options.axis_ring_rescue_table_depth,
            0,
            &config,
        ) {
            Ok(artifacts) => artifacts,
            Err(err) => {
                truncated = true;
                reason = format!("axis_build_failed:{err}");
                break 'axis_loop;
            }
        };
        nodes += axis_artifacts.build_nodes;
        max_suffix = max_suffix.max(
            axis_artifacts
                .table
                .values()
                .map(Vec::len)
                .max()
                .unwrap_or_default(),
        );
        states.extend(axis_artifacts.target_colors);
        if limits.exceeded(nodes) {
            truncated = true;
            reason = limits
                .stop_reason
                .clone()
                .unwrap_or_else(|| "axis-ring seed build stopped".to_string());
            break;
        }
    }

    PdbSeedSet {
        states,
        nodes,
        max_suffix,
        truncated,
        reason,
    }
}

fn collect_ariadne_midpoint_seed_states(puzzle: &Puzzle, options: &BenchOptions) -> PdbSeedSet {
    let mut states = Vec::new();
    let train_scramble_len = options.scramble_lengths.first().copied().unwrap_or(20);
    let train_seed_base = options.seed ^ 0xa71a_dae5_2026_u64;
    let start = options.pdb_seed_step_start.min(options.pdb_seed_step_end);
    let end = options.pdb_seed_step_start.max(options.pdb_seed_step_end);

    for train_index in 0..options.pdb_seed_count {
        let seed = derive_seed(
            train_seed_base,
            puzzle.layout,
            puzzle.difficulty,
            train_scramble_len,
            train_index,
        );
        let scramble = generate_scramble(
            puzzle,
            train_scramble_len,
            seed,
            options.scramble_profile,
            options.avoid_same_tape,
        );
        let solution = ariadne_solution_moves(puzzle, &scramble);
        let mut colors = puzzle.solved_colors.clone();
        puzzle.apply_moves(&mut colors, &scramble);

        for step in 0..=solution.len() {
            if (start..=end).contains(&step) {
                states.push(colors.clone());
            }
            if let Some(&move_index) = solution.get(step) {
                puzzle.apply_move(&mut colors, move_index);
            }
        }
    }

    PdbSeedSet {
        states,
        nodes: 0,
        max_suffix: 0,
        truncated: false,
        reason: "complete".to_string(),
    }
}

fn collect_random_walk_seed_states(puzzle: &Puzzle, options: &BenchOptions) -> PdbSeedSet {
    let mut states = Vec::new();
    let min_len = options.pdb_random_walk_min.min(options.pdb_random_walk_max);
    let max_len = options.pdb_random_walk_min.max(options.pdb_random_walk_max);
    let span = max_len.saturating_sub(min_len) + 1;
    let train_seed_base = options.seed ^ 0x51a7_e5ed_2026_u64;

    for train_index in 0..options.pdb_seed_count {
        let walk_len = min_len + (train_index % span);
        let seed = derive_seed(
            train_seed_base,
            puzzle.layout,
            puzzle.difficulty,
            walk_len,
            train_index,
        );
        let walk = generate_scramble(
            puzzle,
            walk_len,
            seed,
            options.scramble_profile,
            options.avoid_same_tape,
        );
        let mut colors = puzzle.solved_colors.clone();
        puzzle.apply_moves(&mut colors, &walk);
        states.push(colors);
    }

    PdbSeedSet {
        states,
        nodes: 0,
        max_suffix: 0,
        truncated: false,
        reason: "complete".to_string(),
    }
}

fn evaluate_axis_ring_pdb_on_ariadne_path(
    puzzle: &Puzzle,
    target: TargetMode,
    scramble_len: usize,
    iteration: usize,
    seed: u64,
    axis_table_depth: usize,
    target_expand_depth: usize,
    pdb_depth: usize,
    pdb: &AxisRingPdb,
    seed_source: PdbSeedSource,
    scramble: &[MoveIndex],
) -> AxisRingPdbRecord {
    let ariadne_solution = ariadne_solution_moves(puzzle, scramble);
    let mut colors = puzzle.solved_colors.clone();
    puzzle.apply_moves(&mut colors, scramble);

    let path_states = ariadne_solution.len() + 1;
    let start_hit_distance = pdb.distances.get(&state_key(&colors)).copied();
    let mut hit_count = 0usize;
    let mut first_hit_step = None;
    let mut first_hit_distance = None;
    let mut first_hit_remaining_to_solved = None;
    let mut best_hit_step = None;
    let mut best_hit_distance = None;
    let mut best_hit_remaining_to_solved = None;
    let mut best_prefix_to_axis_ring = None;
    let mut distance_remaining = ScoreDistanceStats::new();

    for step in 0..=ariadne_solution.len() {
        let remaining = ariadne_solution.len() - step;
        let current_distance = pdb.distances.get(&state_key(&colors)).copied();
        let radar_distance = current_distance
            .map(usize::from)
            .unwrap_or(pdb_depth.saturating_add(1));
        distance_remaining.add(radar_distance.min(i32::MAX as usize) as i32, remaining);

        if let Some(distance) = current_distance {
            hit_count += 1;
            first_hit_step.get_or_insert(step);
            first_hit_distance.get_or_insert(distance);
            first_hit_remaining_to_solved.get_or_insert(remaining);

            let prefix_to_axis = step + distance as usize;
            let is_better = best_prefix_to_axis_ring.is_none_or(|current| prefix_to_axis < current);
            if is_better {
                best_prefix_to_axis_ring = Some(prefix_to_axis);
                best_hit_step = Some(step);
                best_hit_distance = Some(distance);
                best_hit_remaining_to_solved = Some(remaining);
            }
        }

        if let Some(&move_index) = ariadne_solution.get(step) {
            puzzle.apply_move(&mut colors, move_index);
        }
    }

    AxisRingPdbRecord {
        layout: puzzle.layout,
        difficulty: puzzle.difficulty,
        target,
        seed_source,
        scramble_len,
        iteration: iteration + 1,
        seed,
        axis_table_depth,
        target_expand_depth,
        pdb_depth,
        target_states: pdb.target_states,
        pdb_states: pdb.distances.len(),
        depth_counts: pdb.depth_counts.clone(),
        build_nodes: pdb.nodes,
        build_elapsed_ms: pdb.elapsed_ms,
        truncated: pdb.truncated,
        reason: pdb.reason.clone(),
        ariadne_unit_len: ariadne_solution.len(),
        path_states,
        hit_count,
        start_hit_distance,
        first_hit_step,
        first_hit_distance,
        first_hit_remaining_to_solved,
        best_hit_step,
        best_hit_distance,
        best_hit_remaining_to_solved,
        best_prefix_to_axis_ring,
        max_axis_suffix: pdb.max_axis_suffix,
        estimated_total_with_max_suffix: if seed_source.has_known_suffix_hint() {
            best_prefix_to_axis_ring.map(|prefix| prefix + pdb.max_axis_suffix)
        } else {
            None
        },
        path_distance_remaining_pearson: distance_remaining.pearson(),
    }
}

fn run_phase_quotient_audit(options: BenchOptions) -> io::Result<()> {
    let depth = options.solver.table_depth_for(LayoutId::E).min(12);
    let max_nodes = options.solver.max_nodes;
    let moduli = [3_u8, 6, 12];

    println!(
        "phase-quotient-audit layouts={} difficulties={} depth={} max_nodes={}",
        join_display(&options.layouts),
        join_display(&options.difficulties),
        depth,
        max_nodes
    );

    for &layout in &options.layouts {
        for &difficulty in &options.difficulties {
            let puzzle = Puzzle::new(layout, difficulty);
            let tapes = puzzle_tape_coords(&puzzle);
            println!(
                "{}-{} stickers={} tapes={} moves={}",
                layout,
                difficulty,
                puzzle.stickers.len(),
                tapes.len(),
                puzzle.moves.len()
            );
            println!(
                "tapes: {}",
                tapes
                    .iter()
                    .copied()
                    .map(tape_name)
                    .collect::<Vec<_>>()
                    .join(",")
            );
            print_macro_preservation_audit(&puzzle);

            for modulus in moduli {
                let result =
                    audit_phase_quotient_for_modulus(&puzzle, &tapes, modulus, depth, max_nodes);
                println!(
                    "mod={} explored={} perm_states={} color_states={} truncated={} elapsed={}ms",
                    modulus,
                    result.explored,
                    result.permutation_states,
                    result.color_states,
                    result.truncated,
                    result.elapsed_ms
                );
                if let Some(conflict) = result.permutation_conflict {
                    print_phase_audit_conflict(&puzzle, &tapes, modulus, "permutation", conflict);
                } else {
                    println!(
                        "  permutation_phase_conflict: none found up to depth {}",
                        depth
                    );
                }
                if let Some(conflict) = result.color_conflict {
                    print_phase_audit_conflict(&puzzle, &tapes, modulus, "color", conflict);
                } else {
                    println!("  color_phase_conflict: none found up to depth {}", depth);
                }
            }
        }
    }

    Ok(())
}

fn run_macro_subgroup_audit(options: BenchOptions) -> io::Result<()> {
    let depth = options.solver.table_depth_for(LayoutId::E).min(16);
    let max_nodes = options.solver.max_nodes;
    println!(
        "macro-subgroup-audit layouts={} difficulties={} depth={} max_nodes={} shifts={}",
        join_display(&options.layouts),
        join_display(&options.difficulties),
        depth,
        max_nodes,
        options
            .macro_shifts
            .iter()
            .map(|shift| shift.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );

    for &layout in &options.layouts {
        for &difficulty in &options.difficulties {
            let puzzle = Puzzle::new(layout, difficulty);
            println!(
                "{}-{} stickers={} moves={}",
                layout,
                difficulty,
                puzzle.stickers.len(),
                puzzle.moves.len()
            );
            for &shift in &options.macro_shifts {
                let result = audit_macro_subgroup(&puzzle, shift, depth, max_nodes);
                println!(
                    "shift={} macros={} explored={} perm_states={} color_states={} exhausted={} depth_limited={} truncated={} elapsed={}ms",
                    result.shift,
                    result.macro_count,
                    result.explored,
                    result.permutation_states,
                    result.color_states,
                    result.exhausted,
                    result.depth_limited,
                    result.truncated,
                    result.elapsed_ms
                );
                println!(
                    "  perm_depth_counts={}",
                    result
                        .depth_counts
                        .iter()
                        .enumerate()
                        .map(|(depth, count)| format!("{depth}:{count}"))
                        .collect::<Vec<_>>()
                        .join(",")
                );
                println!(
                    "  color_depth_counts={}",
                    result
                        .color_depth_counts
                        .iter()
                        .enumerate()
                        .map(|(depth, count)| format!("{depth}:{count}"))
                        .collect::<Vec<_>>()
                        .join(",")
                );
            }
        }
    }

    Ok(())
}

fn run_exact_shortcut_audit(options: BenchOptions) -> io::Result<()> {
    fs::create_dir_all(&options.out_dir)?;

    let stamp = unix_stamp();
    let csv_path = options
        .out_dir
        .join(format!("exact_shortcut_audit_{stamp}.csv"));
    let md_path = options
        .out_dir
        .join(format!("exact_shortcut_audit_{stamp}.md"));
    let mut records = Vec::new();
    let iterations = iteration_values(&options);

    println!(
        "shortcut-audit target={} scramble_profile={} layouts={} difficulties={} scrambles={:?} iterations={} seed={} table_depth={} forward_depth={} max_nodes={} time_limit_ms={}",
        options.solver.target_mode.label(),
        options.scramble_profile.label(),
        join_display(&options.layouts),
        join_display(&options.difficulties),
        options.scramble_lengths,
        iteration_label(&options),
        options.seed,
        options
            .solver
            .table_depth
            .map_or_else(|| "auto".to_string(), |depth| depth.to_string()),
        options
            .solver
            .forward_depth
            .map_or_else(|| "auto".to_string(), |depth| depth.to_string()),
        options.solver.max_nodes,
        options.solver.time_limit_ms
    );

    for &layout in &options.layouts {
        for &difficulty in &options.difficulties {
            let puzzle = Puzzle::new(layout, difficulty);
            for target_mode in target_variants(options.solver.target_mode) {
                let mut solver = options.solver.clone();
                solver.target_mode = target_mode;
                let table_depth = solver.table_depth_for(layout);
                let forward_depth = solver.forward_depth_for(layout);
                let proof_bound = table_depth + forward_depth;

                let table_started = Instant::now();
                let mut table_nodes = 0u64;
                let mut table_limits = Limits::new(solver.max_nodes, solver.time_limit_ms);
                let Some(table) =
                    build_reverse_table(&puzzle, &solver, &mut table_nodes, &mut table_limits)
                else {
                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        format!(
                            "failed to build exact shortcut table for {}-{} target={}: {}",
                            layout,
                            difficulty,
                            target_mode.label(),
                            table_limits
                                .stop_reason
                                .clone()
                                .unwrap_or_else(|| "stopped".to_string())
                        ),
                    ));
                };
                let table_elapsed_ms = table_started.elapsed().as_millis();
                println!(
                    "prepared {}-{} target={} table_depth={} forward_depth={} proof_bound={} table_states={} table_nodes={} table_time={}ms",
                    layout,
                    difficulty,
                    target_mode.label(),
                    table_depth,
                    forward_depth,
                    proof_bound,
                    table.len(),
                    table_nodes,
                    table_elapsed_ms
                );

                for &scramble_len in &options.scramble_lengths {
                    for &iteration in &iterations {
                        let seed =
                            derive_seed(options.seed, layout, difficulty, scramble_len, iteration);
                        let scramble = generate_scramble(
                            &puzzle,
                            scramble_len,
                            seed,
                            options.scramble_profile,
                            options.avoid_same_tape,
                        );
                        let ariadne_solution = ariadne_solution_moves(&puzzle, &scramble);
                        let ariadne_solution_len = ariadne_solution.len();
                        let mut colors = puzzle.solved_colors.clone();
                        puzzle.apply_moves(&mut colors, &scramble);

                        let search = exact_shortcut_search(
                            &puzzle,
                            &colors,
                            &table,
                            forward_depth,
                            solver.max_nodes,
                            solver.time_limit_ms,
                        );
                        let optimal_len = if search.found { search.moves.len() } else { 0 };
                        let proved_optimal = search.found && search.complete;
                        let shortcut_moves = if search.found {
                            ariadne_solution_len as isize - optimal_len as isize
                        } else {
                            0
                        };
                        let shortcut_ratio = if search.found && ariadne_solution_len > 0 {
                            optimal_len as f64 / ariadne_solution_len as f64
                        } else {
                            0.0
                        };
                        let significant_shortcut = search.found
                            && (optimal_len * 10 <= ariadne_solution_len * 7
                                || shortcut_moves >= 3);
                        let mut final_colors = colors.clone();
                        puzzle.apply_moves(&mut final_colors, &search.moves);
                        let target_ok = !search.found
                            || is_target_solved(
                                &final_colors,
                                &puzzle,
                                acceptance_target(target_mode),
                            );
                        let reason = if !target_ok {
                            "invalid_target".to_string()
                        } else if proved_optimal {
                            "proved_optimal".to_string()
                        } else if search.found {
                            format!("found_bounded:{}", search.reason)
                        } else {
                            search.reason.clone()
                        };

                        let record = ExactShortcutRecord {
                            layout,
                            difficulty,
                            target_mode,
                            scramble_len,
                            iteration: iteration + 1,
                            seed,
                            ariadne_solution_len,
                            table_depth,
                            forward_depth,
                            proof_bound,
                            table_states: table.len(),
                            table_nodes,
                            table_elapsed_ms,
                            found: search.found && target_ok,
                            proved_optimal: proved_optimal && target_ok,
                            optimal_len,
                            shortcut_moves,
                            shortcut_ratio,
                            significant_shortcut,
                            search_nodes: search.nodes,
                            search_elapsed_ms: search.elapsed_ms,
                            forward_seen: search.forward_seen,
                            table_hits: search.table_hits,
                            complete: search.complete,
                            reason,
                            scramble_script: if options.include_scripts {
                                puzzle.moves_text(&scramble)
                            } else {
                                String::new()
                            },
                            optimal_script: if options.include_scripts {
                                puzzle.moves_text(&search.moves)
                            } else {
                                String::new()
                            },
                        };
                        println!(
                            "{}-{} target={} scramble={} iter={} ariadne={} found={} proved={} opt={} shortcut={} ratio={:.3} nodes={} seen={} hits={} reason={}",
                            layout,
                            difficulty,
                            target_mode.label(),
                            scramble_len,
                            iteration + 1,
                            ariadne_solution_len,
                            record.found,
                            record.proved_optimal,
                            record.optimal_len,
                            record.shortcut_moves,
                            record.shortcut_ratio,
                            record.search_nodes,
                            record.forward_seen,
                            record.table_hits,
                            record.reason
                        );
                        records.push(record);
                    }
                }
            }
        }
    }

    write_exact_shortcut_csv(&csv_path, &records)?;
    write_exact_shortcut_markdown(&md_path, &options, &records)?;
    println!("CSV: {}", csv_path.display());
    println!("MD:  {}", md_path.display());
    Ok(())
}

fn exact_shortcut_search(
    puzzle: &Puzzle,
    colors: &[Color],
    table: &HashMap<Key, PathBits>,
    forward_depth: usize,
    max_nodes: u64,
    time_limit_ms: u64,
) -> ExactShortcutSearchResult {
    let started = Instant::now();
    let mut limits = Limits::new(max_nodes, time_limit_ms);
    let mut nodes = 0u64;
    let mut table_hits = 0usize;
    let start_key = state_key(colors);
    if let Some(path) = table.get(&start_key) {
        return ExactShortcutSearchResult {
            found: true,
            moves: path.to_vec(),
            nodes,
            elapsed_ms: started.elapsed().as_millis(),
            forward_seen: 1,
            table_hits: 1,
            complete: true,
            reason: "start_in_table".to_string(),
        };
    }

    let mut best_hit: Option<Vec<MoveIndex>> = None;
    let mut seen = HashSet::new();
    seen.insert(start_key);
    let mut frontier = vec![ExactEntry {
        colors: colors.to_vec(),
        path: PathBits::default(),
        last_move: None,
    }];

    for _depth in 0..forward_depth {
        let mut next = Vec::new();
        for entry in frontier {
            for move_index in 0..puzzle.moves.len() {
                if is_immediate_inverse(puzzle, entry.last_move, move_index) {
                    continue;
                }
                let next_prefix_len = entry.path.len as usize + 1;
                if best_hit
                    .as_ref()
                    .is_some_and(|best| next_prefix_len >= best.len())
                {
                    continue;
                }

                let mut next_colors = entry.colors.clone();
                puzzle.apply_move(&mut next_colors, move_index);
                nodes += 1;
                if limits.exceeded(nodes) {
                    return ExactShortcutSearchResult {
                        found: best_hit.is_some(),
                        moves: best_hit.unwrap_or_default(),
                        nodes,
                        elapsed_ms: started.elapsed().as_millis(),
                        forward_seen: seen.len(),
                        table_hits,
                        complete: false,
                        reason: limits
                            .stop_reason
                            .clone()
                            .unwrap_or_else(|| "stopped".to_string()),
                    };
                }

                let key = state_key(&next_colors);
                if seen.contains(&key) {
                    continue;
                }

                let path = entry.path.append(move_index);
                if let Some(suffix) = table.get(&key) {
                    table_hits += 1;
                    let mut moves = path.to_vec();
                    moves.extend(suffix.to_vec());
                    keep_shortest(&mut best_hit, moves);
                    continue;
                }

                seen.insert(key);
                next.push(ExactEntry {
                    colors: next_colors,
                    path,
                    last_move: Some(move_index),
                });
            }
        }
        frontier = next;
        if frontier.is_empty() {
            break;
        }
    }

    ExactShortcutSearchResult {
        found: best_hit.is_some(),
        moves: best_hit.unwrap_or_default(),
        nodes,
        elapsed_ms: started.elapsed().as_millis(),
        forward_seen: seen.len(),
        table_hits,
        complete: true,
        reason: if table_hits > 0 {
            "complete_with_hit".to_string()
        } else {
            "complete_no_hit".to_string()
        },
    }
}

fn run_macro_two_stage(options: BenchOptions) -> io::Result<()> {
    fs::create_dir_all(&options.out_dir)?;

    let stamp = unix_stamp();
    let csv_path = options
        .out_dir
        .join(format!("macro6_two_stage_{stamp}.csv"));
    let mut records = Vec::new();
    let iterations = iteration_values(&options);
    let shift = options
        .macro_shifts
        .iter()
        .copied()
        .find(|&candidate| candidate == 6)
        .unwrap_or(6);
    let table_depth = options.macro_table_depth.unwrap_or(12).min(12);

    println!(
        "macro6-two-stage target={} scramble_profile={} layouts={} difficulties={} scrambles={:?} iterations={} iteration_start={} seed={} shift={} table_depth={} pattern_depth={} require_suffix={} projection={} ops={}",
        options.solver.target_mode.label(),
        options.scramble_profile.label(),
        join_display(&options.layouts),
        join_display(&options.difficulties),
        options.scramble_lengths,
        iteration_label(&options),
        options.iteration_start + 1,
        options.seed,
        shift,
        table_depth,
        options.macro_pattern_depth,
        options.macro_require_suffix,
        options.solver.pattern_db_projection.label(),
        options.solver.operation_profile.for_layout(LayoutId::E).label()
    );

    for &layout in &options.layouts {
        for &difficulty in &options.difficulties {
            let puzzle = Puzzle::new(layout, difficulty);
            let target_mode = match options.solver.target_mode {
                TargetMode::PairRegion => TargetMode::Android,
                TargetMode::AndroidPortfolio => TargetMode::Android,
                other => other,
            };
            let artifacts = build_macro_target_artifacts(
                &puzzle,
                target_mode,
                shift,
                table_depth,
                options.macro_pattern_depth,
                &options.solver,
            )?;
            let operations =
                build_operations(&puzzle, options.solver.operation_profile.for_layout(layout));

            println!(
                "prepared {layout}-{difficulty}: macro_states={} target_colors={} macro_ops={} depth_counts={} build={}ms build_nodes={} projection_keys={} projection_build={}ms projection_nodes={} operations={}",
                artifacts.table.len(),
                artifacts.target_colors.len(),
                artifacts.macro_ops.len(),
                artifacts
                    .depth_counts
                    .iter()
                    .enumerate()
                    .map(|(depth, count)| format!("{depth}:{count}"))
                    .collect::<Vec<_>>()
                    .join(","),
                artifacts.build_ms,
                artifacts.build_nodes,
                artifacts
                    .projection_db
                    .as_ref()
                    .map_or(0, PatternDb::direct_len),
                artifacts.projection_build_ms,
                artifacts.projection_build_nodes,
                operations.len()
            );

            for &scramble_len in &options.scramble_lengths {
                for &iteration in &iterations {
                    let run_seed =
                        derive_seed(options.seed, layout, difficulty, scramble_len, iteration);
                    let scramble = generate_scramble(
                        &puzzle,
                        scramble_len,
                        run_seed,
                        options.scramble_profile,
                        options.avoid_same_tape,
                    );
                    let ariadne_solution_len = ariadne_solution_len(&puzzle, &scramble);
                    let mut scrambled_colors = puzzle.solved_colors.clone();
                    puzzle.apply_moves(&mut scrambled_colors, &scramble);

                    let started = Instant::now();
                    let attempt = solve_macro_two_stage_hit(
                        &puzzle,
                        &scrambled_colors,
                        &artifacts,
                        &operations,
                        &options.solver,
                        options.macro_require_suffix,
                    );

                    let mut found = false;
                    let mut reason = attempt.reason.clone();
                    let mut prefix_len = 0usize;
                    let mut suffix_macro_len = 0usize;
                    let mut suffix_unit_len = 0usize;
                    let mut raw_moves = Vec::new();
                    let mut optimized_moves = Vec::new();
                    let nodes = attempt.nodes;

                    if let Some(hit) = attempt.hit {
                        prefix_len = hit.prefix.len();
                        suffix_macro_len = hit.suffix_macro.len as usize;
                        let suffix_units =
                            expand_macro_suffix(&artifacts.macro_ops, hit.suffix_macro);
                        suffix_unit_len = suffix_units.len();
                        raw_moves = hit.prefix;
                        raw_moves.extend(suffix_units);
                        optimized_moves = if options.solver.optimize {
                            optimize_solution(&puzzle, &scrambled_colors, &raw_moves)
                        } else {
                            raw_moves.clone()
                        };
                        found = solution_matches_target(
                            &puzzle,
                            &scrambled_colors,
                            &optimized_moves,
                            acceptance_target(options.solver.target_mode),
                        );
                        reason = if found {
                            hit.reason
                        } else {
                            format!("{}:invalid_target", hit.reason)
                        };
                    }

                    let mut opt_colors = scrambled_colors.clone();
                    puzzle.apply_moves(&mut opt_colors, &optimized_moves);
                    let android_solved =
                        is_android_solved(&opt_colors, &puzzle.face_indexes, difficulty);
                    let elapsed_ms = started.elapsed().as_millis();

                    println!(
                        "{layout}-{difficulty} scramble={scramble_len} iter={} found={} prefix={} suffix_macro={} raw={} opt={} android={} time={}ms nodes={} reason={}",
                        iteration + 1,
                        found,
                        prefix_len,
                        suffix_macro_len,
                        raw_moves.len(),
                        optimized_moves.len(),
                        android_solved,
                        elapsed_ms,
                        nodes,
                        reason
                    );

                    records.push(MacroTwoStageRecord {
                        layout,
                        difficulty,
                        scramble_len,
                        iteration: iteration + 1,
                        seed: run_seed,
                        ariadne_solution_len,
                        found,
                        reason,
                        prefix_len,
                        suffix_macro_len,
                        suffix_unit_len,
                        raw_solution_len: raw_moves.len(),
                        optimized_solution_len: optimized_moves.len(),
                        android_solved,
                        nodes,
                        elapsed_ms,
                        scramble_script: if options.include_scripts {
                            puzzle.moves_text(&scramble)
                        } else {
                            String::new()
                        },
                        raw_solution_script: if options.include_scripts {
                            puzzle.moves_text(&raw_moves)
                        } else {
                            String::new()
                        },
                        optimized_solution_script: if options.include_scripts {
                            puzzle.moves_text(&optimized_moves)
                        } else {
                            String::new()
                        },
                    });
                }
            }
        }
    }

    write_macro_two_stage_csv(&csv_path, &records)?;
    println!("wrote {}", csv_path.display());
    Ok(())
}

fn run_axis_ring_audit(options: BenchOptions) -> io::Result<()> {
    let depth = options.macro_table_depth.unwrap_or(24);
    let target_mode = restricted_seed_target(options.solver.target_mode);
    println!(
        "axis-ring-audit target={} layouts={} difficulties={} depth={} max_nodes={}",
        target_mode.label(),
        join_display(&options.layouts),
        join_display(&options.difficulties),
        depth,
        options.solver.max_nodes
    );

    for &layout in &options.layouts {
        for &difficulty in &options.difficulties {
            let puzzle = Puzzle::new(layout, difficulty);
            for axis in 0..3_u8 {
                let moves = axis_moves(&puzzle, axis);
                if moves.is_empty() {
                    continue;
                }
                let mut nodes = 0u64;
                let mut limits =
                    Limits::new(options.solver.max_nodes, options.solver.time_limit_ms);
                let started = Instant::now();
                let (table, _, depth_counts) = build_restricted_color_table(
                    &puzzle,
                    target_mode,
                    &moves,
                    depth,
                    &mut nodes,
                    &mut limits,
                )?;
                let max_suffix_len = table.values().map(Vec::len).max().unwrap_or_default();
                println!(
                    "{}-{} axis={} fixed_faces={} moves={} states={} max_suffix={} nodes={} elapsed={}ms depth_counts={}",
                    layout,
                    difficulty,
                    axis_label(axis),
                    axis_fixed_faces_label(axis),
                    moves.len(),
                    table.len(),
                    max_suffix_len,
                    nodes,
                    started.elapsed().as_millis(),
                    join_usize_counts(&depth_counts)
                );
            }
        }
    }

    Ok(())
}

fn run_middle_subgroup_audit(options: BenchOptions) -> io::Result<()> {
    let depth = options.macro_table_depth.unwrap_or(12);
    let max_nodes = options.solver.max_nodes;
    println!(
        "middle-subgroup-audit layouts={} difficulties={} depth={} max_nodes={}",
        join_display(&options.layouts),
        join_display(&options.difficulties),
        depth,
        max_nodes
    );

    for &layout in &options.layouts {
        for &difficulty in &options.difficulties {
            let puzzle = Puzzle::new(layout, difficulty);
            let moves = middle_axis_moves(&puzzle);
            let result = audit_restricted_subgroup(&puzzle, &moves, depth, max_nodes);
            println!(
                "{}-{} moves={} explored={} perm_states={} color_states={} exhausted={} depth_limited={} truncated={} elapsed={}ms",
                layout,
                difficulty,
                moves.len(),
                result.explored,
                result.permutation_states,
                result.color_states,
                result.exhausted,
                result.depth_limited,
                result.truncated,
                result.elapsed_ms
            );
            println!(
                "  perm_depth_counts={}",
                join_usize_counts(&result.depth_counts)
            );
            println!(
                "  color_depth_counts={}",
                join_usize_counts(&result.color_depth_counts)
            );
        }
    }

    Ok(())
}

fn run_axis_ring_two_stage(options: BenchOptions) -> io::Result<()> {
    fs::create_dir_all(&options.out_dir)?;

    let stamp = unix_stamp();
    let csv_path = options
        .out_dir
        .join(format!("axis_ring_two_stage_{stamp}.csv"));
    let mut records = Vec::new();
    let iterations = iteration_values(&options);
    let target_mode = restricted_seed_target(options.solver.target_mode);
    let table_depth = options.macro_table_depth.unwrap_or(24);

    println!(
        "axis-ring-two-stage target={} scramble_profile={} layouts={} difficulties={} scrambles={:?} iterations={} iteration_start={} seed={} table_depth={} target_expand_depth={} pattern_depth={} require_suffix={} projection={} axis_profile_weight={} axis_order_weight={} ops={}",
        target_mode.label(),
        options.scramble_profile.label(),
        join_display(&options.layouts),
        join_display(&options.difficulties),
        options.scramble_lengths,
        iteration_label(&options),
        options.iteration_start + 1,
        options.seed,
        table_depth,
        options.solver.target_expand_depth,
        options.macro_pattern_depth,
        options.macro_require_suffix,
        options.solver.pattern_db_projection.label(),
        options.solver.axis_ring_profile_weight,
        options.solver.axis_ring_order_weight,
        options.solver.operation_profile.for_layout(LayoutId::E).label()
    );

    for &layout in &options.layouts {
        for &difficulty in &options.difficulties {
            let puzzle = Puzzle::new(layout, difficulty);
            let operations =
                build_operations(&puzzle, options.solver.operation_profile.for_layout(layout));
            let mut optimizer_config = options.solver.clone();
            optimizer_config.target_mode = target_mode;
            let optimizer_artifacts = if options.solver.optimize {
                Some(prepare_solver(&puzzle, &optimizer_config)?)
            } else {
                None
            };
            let optimizer_table = optimizer_artifacts
                .as_ref()
                .and_then(|artifacts| {
                    artifacts
                        .variants
                        .iter()
                        .find(|variant| variant.target_mode == target_mode)
                        .or_else(|| artifacts.variants.first())
                })
                .map(|variant| &variant.table);
            if let Some(table) = optimizer_table {
                println!(
                    "prepared {layout}-{difficulty} optimizer target={} states={}",
                    target_mode.label(),
                    table.len()
                );
            }
            let mut axis_artifacts = Vec::new();
            for axis in 0..3_u8 {
                let moves = axis_moves(&puzzle, axis);
                if moves.is_empty() {
                    continue;
                }
                let artifacts = build_restricted_target_artifacts(
                    &puzzle,
                    target_mode,
                    moves,
                    table_depth,
                    options.macro_pattern_depth,
                    &options.solver,
                )?;
                println!(
                    "prepared {layout}-{difficulty} axis={} fixed_faces={} states={} target_colors={} restricted_moves={} max_suffix={} build={}ms build_nodes={} projection_keys={} projection_build={}ms projection_nodes={} operations={} depth_counts={}",
                    axis_label(axis),
                    axis_fixed_faces_label(axis),
                    artifacts.table.len(),
                    artifacts.target_colors.len(),
                    artifacts.allowed_moves.len(),
                    artifacts.table.values().map(Vec::len).max().unwrap_or_default(),
                    artifacts.build_ms,
                    artifacts.build_nodes,
                    artifacts.projection_db.as_ref().map_or(0, PatternDb::direct_len),
                    artifacts.projection_build_ms,
                    artifacts.projection_build_nodes,
                    operations.len(),
                    join_usize_counts(&artifacts.depth_counts)
                );
                axis_artifacts.push((axis, artifacts));
            }

            for &scramble_len in &options.scramble_lengths {
                for &iteration in &iterations {
                    let run_seed =
                        derive_seed(options.seed, layout, difficulty, scramble_len, iteration);
                    let scramble = generate_scramble(
                        &puzzle,
                        scramble_len,
                        run_seed,
                        options.scramble_profile,
                        options.avoid_same_tape,
                    );
                    let ariadne_solution_len = ariadne_solution_len(&puzzle, &scramble);
                    let mut scrambled_colors = puzzle.solved_colors.clone();
                    puzzle.apply_moves(&mut scrambled_colors, &scramble);

                    let mut best_record: Option<AxisRingRecord> = None;
                    for (axis, artifacts) in &axis_artifacts {
                        let started = Instant::now();
                        let attempt = solve_restricted_target_hit(
                            &puzzle,
                            &scrambled_colors,
                            artifacts,
                            &operations,
                            &options.solver,
                            options.macro_require_suffix,
                        );

                        let mut found = false;
                        let mut reason = attempt.reason.clone();
                        let mut prefix_len = 0usize;
                        let mut suffix_len = 0usize;
                        let mut raw_moves = Vec::new();
                        let mut optimized_moves = Vec::new();
                        let nodes = attempt.nodes;
                        if let Some(hit) = attempt.hit {
                            prefix_len = hit.prefix.len();
                            suffix_len = hit.suffix.len();
                            raw_moves = hit.prefix;
                            raw_moves.extend(hit.suffix);
                            optimized_moves = if options.solver.optimize {
                                if let Some(table) = optimizer_table {
                                    let (local_window, local_depth) =
                                        local_optimization_for_profile(
                                            &puzzle,
                                            &options.solver,
                                            options.solver.operation_profile.for_layout(layout),
                                        );
                                    optimize_solution_with_table(
                                        &puzzle,
                                        &scrambled_colors,
                                        &raw_moves,
                                        table,
                                        local_window,
                                        local_depth,
                                    )
                                } else {
                                    optimize_solution(&puzzle, &scrambled_colors, &raw_moves)
                                }
                            } else {
                                raw_moves.clone()
                            };
                            found = solution_matches_target(
                                &puzzle,
                                &scrambled_colors,
                                &optimized_moves,
                                acceptance_target(target_mode),
                            );
                            reason = if found {
                                hit.reason
                            } else {
                                format!("{}:invalid_target", hit.reason)
                            };
                        }

                        let mut opt_colors = scrambled_colors.clone();
                        puzzle.apply_moves(&mut opt_colors, &optimized_moves);
                        let android_solved =
                            is_android_solved(&opt_colors, &puzzle.face_indexes, difficulty);
                        let record = AxisRingRecord {
                            layout,
                            difficulty,
                            scramble_len,
                            iteration: iteration + 1,
                            seed: run_seed,
                            axis: *axis,
                            ariadne_solution_len,
                            found,
                            reason,
                            prefix_len,
                            suffix_len,
                            raw_solution_len: raw_moves.len(),
                            optimized_solution_len: optimized_moves.len(),
                            android_solved,
                            nodes,
                            elapsed_ms: started.elapsed().as_millis(),
                            scramble_script: if options.include_scripts {
                                puzzle.moves_text(&scramble)
                            } else {
                                String::new()
                            },
                            raw_solution_script: if options.include_scripts {
                                puzzle.moves_text(&raw_moves)
                            } else {
                                String::new()
                            },
                            optimized_solution_script: if options.include_scripts {
                                puzzle.moves_text(&optimized_moves)
                            } else {
                                String::new()
                            },
                        };
                        keep_best_axis_ring_record(&mut best_record, record);
                    }

                    let record = best_record.expect("at least one axis target");
                    println!(
                        "{layout}-{difficulty} scramble={scramble_len} iter={} axis={} found={} prefix={} suffix={} raw={} opt={} android={} time={}ms nodes={} reason={}",
                        iteration + 1,
                        axis_label(record.axis),
                        record.found,
                        record.prefix_len,
                        record.suffix_len,
                        record.raw_solution_len,
                        record.optimized_solution_len,
                        record.android_solved,
                        record.elapsed_ms,
                        record.nodes,
                        record.reason
                    );
                    records.push(record);
                }
            }
        }
    }

    write_axis_ring_csv(&csv_path, &records)?;
    println!("wrote {}", csv_path.display());
    Ok(())
}

#[derive(Debug, Clone)]
struct PhaseQuotientAuditResult {
    explored: u64,
    permutation_states: usize,
    color_states: usize,
    truncated: bool,
    elapsed_ms: u128,
    permutation_conflict: Option<PhaseAuditConflict>,
    color_conflict: Option<PhaseAuditConflict>,
}

fn audit_phase_quotient_for_modulus(
    puzzle: &Puzzle,
    tapes: &[TapeCoord],
    modulus: u8,
    max_depth: usize,
    max_nodes: u64,
) -> PhaseQuotientAuditResult {
    let started = Instant::now();
    let identity_permutation = (0..puzzle.stickers.len())
        .map(|index| index as u8)
        .collect::<Vec<_>>();
    let mut permutation_seen: HashMap<Vec<u8>, PhaseAuditEntry> = HashMap::new();
    let mut color_seen: HashMap<Key, PhaseAuditEntry> = HashMap::new();
    let mut queue = VecDeque::new();

    let root = PhaseAuditNode {
        permutation: identity_permutation.clone(),
        colors: puzzle.solved_colors.clone(),
        phase: 0,
        path: PathBits::default(),
        last_move: None,
    };
    permutation_seen.insert(
        identity_permutation,
        PhaseAuditEntry {
            phase: 0,
            path: PathBits::default(),
        },
    );
    color_seen.insert(
        state_key(&puzzle.solved_colors),
        PhaseAuditEntry {
            phase: 0,
            path: PathBits::default(),
        },
    );
    queue.push_back(root);

    let mut explored = 0u64;
    let mut truncated = false;
    let mut permutation_conflict = None;
    let mut color_conflict = None;

    while let Some(node) = queue.pop_front() {
        if explored >= max_nodes {
            truncated = true;
            break;
        }
        explored += 1;
        if node.path.len as usize >= max_depth {
            continue;
        }

        for move_index in 0..puzzle.moves.len() {
            if is_immediate_inverse(puzzle, node.last_move, move_index) {
                continue;
            }

            let mut next_permutation = node.permutation.clone();
            apply_move_to_permutation(puzzle, &mut next_permutation, move_index);

            let mut next_colors = node.colors.clone();
            puzzle.apply_move(&mut next_colors, move_index);

            let next_phase = update_phase_vector(puzzle, tapes, node.phase, move_index, modulus);
            let next_path = node.path.append(move_index);
            let next_last_move = Some(move_index);

            if permutation_conflict.is_none() {
                if let Some(existing) = permutation_seen.get(&next_permutation) {
                    if existing.phase != next_phase {
                        permutation_conflict = Some(PhaseAuditConflict {
                            key_kind: "permutation",
                            existing_phase: existing.phase,
                            new_phase: next_phase,
                            existing_path: existing.path,
                            new_path: next_path,
                        });
                    }
                }
            }

            let color_key = state_key(&next_colors);
            if color_conflict.is_none() {
                if let Some(existing) = color_seen.get(&color_key) {
                    if existing.phase != next_phase {
                        color_conflict = Some(PhaseAuditConflict {
                            key_kind: "color",
                            existing_phase: existing.phase,
                            new_phase: next_phase,
                            existing_path: existing.path,
                            new_path: next_path,
                        });
                    }
                }
            }

            let is_new_permutation = match permutation_seen.entry(next_permutation.clone()) {
                std::collections::hash_map::Entry::Occupied(_) => false,
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert(PhaseAuditEntry {
                        phase: next_phase,
                        path: next_path,
                    });
                    true
                }
            };
            color_seen.entry(color_key).or_insert(PhaseAuditEntry {
                phase: next_phase,
                path: next_path,
            });

            if is_new_permutation {
                queue.push_back(PhaseAuditNode {
                    permutation: next_permutation,
                    colors: next_colors,
                    phase: next_phase,
                    path: next_path,
                    last_move: next_last_move,
                });
            }
        }

        if permutation_conflict.is_some() && color_conflict.is_some() {
            break;
        }
    }

    PhaseQuotientAuditResult {
        explored,
        permutation_states: permutation_seen.len(),
        color_states: color_seen.len(),
        truncated,
        elapsed_ms: started.elapsed().as_millis(),
        permutation_conflict,
        color_conflict,
    }
}

fn audit_puzzle_heuristics(puzzle: &Puzzle, config: &SolverConfig) -> HeuristicAuditRecord {
    let started = Instant::now();
    let depth = config.pattern_db_depth;
    let mut limits = Limits::new(config.max_nodes, config.time_limit_ms);
    let mut nodes = 0u64;
    let mut seen: HashSet<Key> = HashSet::new();
    let mut histogram_keys: HashSet<PatternKey> = HashSet::new();
    let mut canonical_histogram_keys: HashSet<PatternKey> = HashSet::new();
    let mut score_distance = ScoreDistanceStats::new();
    let mut depth_counts = vec![0usize; depth + 1];
    let mut depth_score_sums = vec![0f64; depth + 1];
    let mut frontier = Vec::new();
    let mut truncated = false;
    let mut reason = "complete".to_string();

    for seed in reverse_table_seeds(puzzle, config.target_mode) {
        let key = state_key(&seed);
        if seen.insert(key) {
            record_heuristic_observation(
                puzzle,
                config,
                &seed,
                0,
                &mut histogram_keys,
                &mut canonical_histogram_keys,
                &mut score_distance,
                &mut depth_counts,
                &mut depth_score_sums,
            );
            frontier.push(PatternEntry {
                colors: seed,
                last_move: None,
            });
        }
    }

    'depth_loop: for current_depth in 0..depth {
        let mut next = Vec::new();
        let next_distance = current_depth + 1;
        for entry in frontier {
            for move_index in 0..puzzle.moves.len() {
                if is_immediate_inverse(puzzle, entry.last_move, move_index) {
                    continue;
                }

                let mut colors = entry.colors.clone();
                puzzle.apply_move(&mut colors, move_index);
                nodes += 1;
                if limits.exceeded(nodes) {
                    truncated = true;
                    reason = limits
                        .stop_reason
                        .clone()
                        .unwrap_or_else(|| "stopped".to_string());
                    break 'depth_loop;
                }

                let key = state_key(&colors);
                if !seen.insert(key) {
                    continue;
                }

                record_heuristic_observation(
                    puzzle,
                    config,
                    &colors,
                    next_distance,
                    &mut histogram_keys,
                    &mut canonical_histogram_keys,
                    &mut score_distance,
                    &mut depth_counts,
                    &mut depth_score_sums,
                );
                next.push(PatternEntry {
                    colors,
                    last_move: Some(move_index),
                });
            }
        }
        frontier = next;
        if frontier.is_empty() {
            break;
        }
    }

    let mean_score_by_depth = depth_counts
        .iter()
        .zip(depth_score_sums.iter())
        .map(
            |(&count, &sum)| {
                if count == 0 {
                    0.0
                } else {
                    sum / count as f64
                }
            },
        )
        .collect::<Vec<_>>();

    HeuristicAuditRecord {
        layout: puzzle.layout,
        difficulty: puzzle.difficulty,
        target: config.target_mode,
        depth,
        states: seen.len(),
        histogram_keys: histogram_keys.len(),
        canonical_histogram_keys: canonical_histogram_keys.len(),
        nodes,
        elapsed_ms: started.elapsed().as_millis(),
        pearson_score_distance: score_distance.pearson(),
        truncated,
        reason,
        depth_counts,
        mean_score_by_depth,
    }
}

fn audit_puzzle_projection(
    puzzle: &Puzzle,
    config: &SolverConfig,
    projection: ProjectionKind,
) -> ProjectionAuditRecord {
    let started = Instant::now();
    let depth = config.pattern_db_depth;
    let mut limits = Limits::new(config.max_nodes, config.time_limit_ms);
    let mut nodes = 0u64;
    let mut seen: HashSet<Key> = HashSet::new();
    let mut buckets: HashMap<Vec<u8>, ProjectionBucket> = HashMap::new();
    let mut frontier = Vec::new();
    let mut truncated = false;
    let mut reason = "complete".to_string();

    for seed in reverse_table_seeds(puzzle, config.target_mode) {
        let key = state_key(&seed);
        if seen.insert(key) {
            record_projection_observation(puzzle, &seed, 0, projection, &mut buckets);
            frontier.push(PatternEntry {
                colors: seed,
                last_move: None,
            });
        }
    }

    'depth_loop: for current_depth in 0..depth {
        let mut next = Vec::new();
        let next_distance = current_depth + 1;
        for entry in frontier {
            for move_index in 0..puzzle.moves.len() {
                if is_immediate_inverse(puzzle, entry.last_move, move_index) {
                    continue;
                }

                let mut colors = entry.colors.clone();
                puzzle.apply_move(&mut colors, move_index);
                nodes += 1;
                if limits.exceeded(nodes) {
                    truncated = true;
                    reason = limits
                        .stop_reason
                        .clone()
                        .unwrap_or_else(|| "stopped".to_string());
                    break 'depth_loop;
                }

                let key = state_key(&colors);
                if !seen.insert(key) {
                    continue;
                }

                record_projection_observation(
                    puzzle,
                    &colors,
                    next_distance,
                    projection,
                    &mut buckets,
                );
                next.push(PatternEntry {
                    colors,
                    last_move: Some(move_index),
                });
            }
        }
        frontier = next;
        if frontier.is_empty() {
            break;
        }
    }

    let depth_spans = buckets
        .values()
        .map(ProjectionBucket::depth_span)
        .collect::<Vec<_>>();
    let bucket_sizes = buckets
        .values()
        .map(|bucket| bucket.count)
        .collect::<Vec<_>>();

    ProjectionAuditRecord {
        layout: puzzle.layout,
        difficulty: puzzle.difficulty,
        target: config.target_mode,
        projection,
        depth,
        states: seen.len(),
        keys: buckets.len(),
        mean_depth_span: mean_usize(&depth_spans).unwrap_or_default(),
        p95_depth_span: percentile_usize(&depth_spans, 0.95).unwrap_or_default(),
        max_depth_span: depth_spans.iter().copied().max().unwrap_or_default(),
        mean_states_per_key: mean_usize(&bucket_sizes).unwrap_or_default(),
        p95_states_per_key: percentile_usize(&bucket_sizes, 0.95).unwrap_or_default(),
        max_states_per_key: bucket_sizes.iter().copied().max().unwrap_or_default(),
        nodes,
        elapsed_ms: started.elapsed().as_millis(),
        truncated,
        reason,
    }
}

fn record_heuristic_observation(
    puzzle: &Puzzle,
    config: &SolverConfig,
    colors: &[Color],
    distance: usize,
    histogram_keys: &mut HashSet<PatternKey>,
    canonical_histogram_keys: &mut HashSet<PatternKey>,
    score_distance: &mut ScoreDistanceStats,
    depth_counts: &mut [usize],
    depth_score_sums: &mut [f64],
) {
    histogram_keys.insert(face_histogram_key(colors, puzzle));
    canonical_histogram_keys.insert(canonical_face_histogram_key(colors, puzzle));
    let score = score_state(
        colors,
        puzzle,
        config.target_mode,
        config.region_pair_weight,
        None,
    );
    score_distance.add(score, distance);
    if let Some(count) = depth_counts.get_mut(distance) {
        *count += 1;
    }
    if let Some(sum) = depth_score_sums.get_mut(distance) {
        *sum += score as f64;
    }
}

fn record_projection_observation(
    puzzle: &Puzzle,
    colors: &[Color],
    distance: usize,
    projection: ProjectionKind,
    buckets: &mut HashMap<Vec<u8>, ProjectionBucket>,
) {
    let key = projection_key(puzzle, colors, projection);
    match buckets.get_mut(&key) {
        Some(bucket) => bucket.add(distance),
        None => {
            buckets.insert(key, ProjectionBucket::new(distance));
        }
    }
}

fn projection_key(puzzle: &Puzzle, colors: &[Color], projection: ProjectionKind) -> Vec<u8> {
    match projection {
        ProjectionKind::FaceHistogram => face_histogram_key(colors, puzzle).to_vec(),
        ProjectionKind::CanonicalFaceHistogram => {
            canonical_face_histogram_key(colors, puzzle).to_vec()
        }
        ProjectionKind::AndroidPairFaces => android_pair_faces_key(puzzle, colors),
        ProjectionKind::FaceHomeMatches => face_home_matches_key(puzzle, colors),
        ProjectionKind::FacePairHomeMatches => face_pair_home_matches_key(puzzle, colors),
        ProjectionKind::TapeHistogram => tape_histogram_key(puzzle, colors),
        ProjectionKind::TapePairHistogram => tape_pair_histogram_key(puzzle, colors),
        ProjectionKind::TapeQuality => tape_quality_key(puzzle, colors),
        ProjectionKind::TapePairQuality => tape_pair_quality_key(puzzle, colors),
        ProjectionKind::TapeSegments => tape_segments_key(puzzle, colors),
        ProjectionKind::TapePairSegments => tape_pair_segments_key(puzzle, colors),
    }
}

fn android_pair_faces_key(puzzle: &Puzzle, colors: &[Color]) -> Vec<u8> {
    let required_pairs = puzzle.difficulty.required_pairs();
    let mut key = Vec::with_capacity(FACE_COUNT * required_pairs.len());
    for face_indexes in &puzzle.face_indexes {
        for &(left, right) in &required_pairs {
            let mut count = 0_u8;
            for &index in face_indexes {
                if colors[index] == left || colors[index] == right {
                    count = count.saturating_add(1);
                }
            }
            key.push(count);
        }
    }
    key
}

fn face_home_matches_key(puzzle: &Puzzle, colors: &[Color]) -> Vec<u8> {
    let mut key = Vec::with_capacity(FACE_COUNT);
    for (face_index, face_indexes) in puzzle.face_indexes.iter().enumerate() {
        let target = puzzle.difficulty.face_color(FACES[face_index]);
        let count = face_indexes
            .iter()
            .filter(|&&index| colors[index] == target)
            .count();
        key.push(count as u8);
    }
    key
}

fn face_pair_home_matches_key(puzzle: &Puzzle, colors: &[Color]) -> Vec<u8> {
    let mut key = Vec::with_capacity(FACE_COUNT);
    for (face_index, face_indexes) in puzzle.face_indexes.iter().enumerate() {
        let target_pair =
            android_pair_index(puzzle, puzzle.difficulty.face_color(FACES[face_index]));
        let count = face_indexes
            .iter()
            .filter(|&&index| android_pair_index(puzzle, colors[index]) == target_pair)
            .count();
        key.push(count as u8);
    }
    key
}

fn tape_coords(puzzle: &Puzzle) -> Vec<TapeCoord> {
    let mut coords = puzzle
        .moves
        .iter()
        .filter(|mv| mv.direction > 0)
        .map(|mv| TapeCoord {
            axis: mv.axis,
            layer: mv.layer,
        })
        .collect::<Vec<_>>();
    coords.sort_unstable();
    coords.dedup();
    coords
}

fn tape_histogram_key(puzzle: &Puzzle, colors: &[Color]) -> Vec<u8> {
    let coords = tape_coords(puzzle);
    let mut key = Vec::with_capacity(coords.len() * 6);
    for coord in coords {
        let mut counts = [0_u8; 6];
        if let Some(cycle) = tape_cycle(puzzle, coord) {
            for &index in cycle {
                counts[colors[index] as usize] = counts[colors[index] as usize].saturating_add(1);
            }
        }
        key.extend(counts);
    }
    key
}

fn android_pair_index(puzzle: &Puzzle, color: Color) -> u8 {
    puzzle
        .difficulty
        .required_pairs()
        .iter()
        .position(|&(left, right)| color == left || color == right)
        .map(|index| index as u8)
        .unwrap_or(255)
}

fn tape_pair_histogram_key(puzzle: &Puzzle, colors: &[Color]) -> Vec<u8> {
    let coords = tape_coords(puzzle);
    let pair_count = puzzle.difficulty.required_pairs().len();
    let mut key = Vec::with_capacity(coords.len() * pair_count);
    for coord in coords {
        let mut counts = vec![0_u8; pair_count];
        if let Some(cycle) = tape_cycle(puzzle, coord) {
            for &index in cycle {
                let pair_index = android_pair_index(puzzle, colors[index]) as usize;
                if let Some(count) = counts.get_mut(pair_index) {
                    *count = count.saturating_add(1);
                }
            }
        }
        key.extend(counts);
    }
    key
}

fn tape_quality_key(puzzle: &Puzzle, colors: &[Color]) -> Vec<u8> {
    let coords = tape_coords(puzzle);
    let mut key = Vec::with_capacity(coords.len() * 3);
    for coord in coords {
        match analyze_tape(puzzle, colors, coord) {
            Some(analysis) => {
                key.push(analysis.segment_misses as u8);
                key.push(analysis.unique_penalty as u8);
                key.push(analysis.pair_misses as u8);
            }
            None => key.extend([255, 255, 255]),
        }
    }
    key
}

fn tape_pair_quality_key(puzzle: &Puzzle, colors: &[Color]) -> Vec<u8> {
    let coords = tape_coords(puzzle);
    let mut key = Vec::with_capacity(coords.len() * 3);
    for coord in coords {
        let Some(cycle) = tape_cycle(puzzle, coord) else {
            key.extend([255, 255, 255]);
            continue;
        };
        if cycle.is_empty() || cycle.len() % 4 != 0 {
            key.extend([255, 255, 255]);
            continue;
        }

        let segment_len = cycle.len() / 4;
        let mut dominant_pairs = [0_u8; 4];
        let mut pair_misses = 0usize;
        let mut unique_pair_penalty = 0usize;
        for segment in 0..4 {
            let mut counts = [0usize; 3];
            for &index in &cycle[segment * segment_len..(segment + 1) * segment_len] {
                let pair_index = android_pair_index(puzzle, colors[index]) as usize;
                if let Some(count) = counts.get_mut(pair_index) {
                    *count += 1;
                }
            }
            let (dominant_pair, dominant_count) = counts
                .iter()
                .copied()
                .enumerate()
                .max_by_key(|&(pair, count)| (count, std::cmp::Reverse(pair)))
                .unwrap_or((0, 0));
            let unique_pairs = counts.iter().filter(|&&count| count > 0).count();
            dominant_pairs[segment] = dominant_pair as u8;
            pair_misses += segment_len.saturating_sub(dominant_count);
            unique_pair_penalty += unique_pairs.saturating_sub(1);
        }
        let opposite_pair_misses = usize::from(dominant_pairs[0] != dominant_pairs[2])
            + usize::from(dominant_pairs[1] != dominant_pairs[3]);
        key.push(pair_misses as u8);
        key.push(unique_pair_penalty as u8);
        key.push(opposite_pair_misses as u8);
    }
    key
}

fn tape_segments_key(puzzle: &Puzzle, colors: &[Color]) -> Vec<u8> {
    let coords = tape_coords(puzzle);
    let mut key = Vec::with_capacity(coords.len() * 13);
    for coord in coords {
        let Some(cycle) = tape_cycle(puzzle, coord) else {
            key.extend([255; 13]);
            continue;
        };
        if cycle.is_empty() || cycle.len() % 4 != 0 {
            key.extend([255; 13]);
            continue;
        }

        let segment_len = cycle.len() / 4;
        for segment in 0..4 {
            let mut counts = [0usize; 6];
            for &index in &cycle[segment * segment_len..(segment + 1) * segment_len] {
                counts[colors[index] as usize] += 1;
            }
            let (dominant_color, dominant_count) = counts
                .iter()
                .copied()
                .enumerate()
                .max_by_key(|&(color, count)| (count, std::cmp::Reverse(color)))
                .unwrap_or((0, 0));
            let unique_colors = counts.iter().filter(|&&count| count > 0).count();
            key.push(dominant_color as u8);
            key.push(segment_len.saturating_sub(dominant_count) as u8);
            key.push(unique_colors as u8);
        }
        key.push(
            analyze_tape(puzzle, colors, coord)
                .map(|analysis| analysis.pair_misses as u8)
                .unwrap_or(255),
        );
    }
    key
}

fn tape_pair_segments_key(puzzle: &Puzzle, colors: &[Color]) -> Vec<u8> {
    let coords = tape_coords(puzzle);
    let pair_count = puzzle.difficulty.required_pairs().len();
    let mut key = Vec::with_capacity(coords.len() * 4 * pair_count);
    for coord in coords {
        let Some(cycle) = tape_cycle(puzzle, coord) else {
            key.extend(vec![255; 4 * pair_count]);
            continue;
        };
        if cycle.is_empty() || cycle.len() % 4 != 0 {
            key.extend(vec![255; 4 * pair_count]);
            continue;
        }

        let segment_len = cycle.len() / 4;
        for segment in 0..4 {
            let mut counts = vec![0_u8; pair_count];
            for &index in &cycle[segment * segment_len..(segment + 1) * segment_len] {
                let pair_index = android_pair_index(puzzle, colors[index]) as usize;
                if let Some(count) = counts.get_mut(pair_index) {
                    *count = count.saturating_add(1);
                }
            }
            key.extend(counts);
        }
    }
    key
}

const MOVE_DELTA_FEATURE_LABELS: [&str; 7] = [
    "score_state",
    "ring_entropy",
    "axis_pair_quality",
    "tape_segment_conflicts",
    "perm_slot_cost",
    "perm_cycle_cost",
    "ring_rotation_miss",
];

#[derive(Debug, Clone, Copy)]
struct MoveDeltaMetrics {
    score: i32,
    ring_entropy: f64,
    axis_pair_quality: i32,
    tape_segment_conflicts: i32,
    perm_slot_cost: i32,
    perm_cycle_cost: i32,
    ring_rotation_miss: i32,
}

#[derive(Debug, Clone, Copy)]
struct MoveDeltaCandidate {
    raw_match: bool,
    first_match: bool,
    values: [f64; 7],
}

#[derive(Debug, Clone, Copy)]
struct MoveDeltaRank {
    rank: f64,
    percentile: f64,
    value: f64,
}

#[derive(Debug, Clone)]
struct DirectionSurvivalBeamEntry {
    colors: Vec<Color>,
    path_len: usize,
    corridor_prefix: Vec<MoveIndex>,
    last_move: Option<MoveIndex>,
    first_move: Option<MoveIndex>,
    score: i32,
    rank_score: i32,
}

#[derive(Debug, Clone)]
struct PrefixSurvivalBeamEntry {
    colors: Vec<Color>,
    path_len: usize,
    corridor_prefix: Vec<MoveIndex>,
    last_move: Option<MoveIndex>,
    matching_prefix_len: usize,
    score: i32,
    rank_score: i32,
}

fn evaluate_move_delta_step(
    puzzle: &Puzzle,
    config: &SolverConfig,
    operation_profile: OperationProfile,
    operations: &[Operation],
    scramble_len: usize,
    iteration: usize,
    seed: u64,
    ariadne_unit_len: usize,
    step: usize,
    ariadne_move: MoveIndex,
    colors: &[Color],
) -> Vec<MoveDeltaAuditRecord> {
    let before = move_delta_metrics(puzzle, config, colors);
    let mut candidates = Vec::with_capacity(operations.len());

    for operation in operations {
        let mut next_colors = colors.to_vec();
        for &move_index in &operation.moves {
            puzzle.apply_move(&mut next_colors, move_index);
        }
        let after = move_delta_metrics(puzzle, config, &next_colors);
        candidates.push(MoveDeltaCandidate {
            raw_match: operation.is_raw
                && operation.moves.len() == 1
                && operation.moves[0] == ariadne_move,
            first_match: operation.moves.first().copied() == Some(ariadne_move),
            values: [
                (before.score - after.score) as f64,
                before.ring_entropy - after.ring_entropy,
                (after.axis_pair_quality - before.axis_pair_quality) as f64,
                (before.tape_segment_conflicts - after.tape_segment_conflicts) as f64,
                (before.perm_slot_cost - after.perm_slot_cost) as f64,
                (before.perm_cycle_cost - after.perm_cycle_cost) as f64,
                (before.ring_rotation_miss - after.ring_rotation_miss) as f64,
            ],
        });
    }

    let first_match_count = candidates
        .iter()
        .filter(|candidate| candidate.first_match)
        .count();
    let step_bucket = move_delta_step_bucket(step);
    let remaining_to_solved = ariadne_unit_len.saturating_sub(step);
    let ariadne_move_label = puzzle.moves_text(&[ariadne_move]);
    let mut records = Vec::with_capacity(MOVE_DELTA_FEATURE_LABELS.len());

    for (feature_index, &feature) in MOVE_DELTA_FEATURE_LABELS.iter().enumerate() {
        let raw_rank = move_delta_rank(&candidates, feature_index, |candidate| candidate.raw_match);
        let first_match_rank = move_delta_rank(&candidates, feature_index, |candidate| {
            candidate.first_match
        });
        let best_delta = candidates
            .iter()
            .map(|candidate| candidate.values[feature_index])
            .fold(f64::NEG_INFINITY, f64::max);
        let worst_delta = candidates
            .iter()
            .map(|candidate| candidate.values[feature_index])
            .fold(f64::INFINITY, f64::min);

        records.push(MoveDeltaAuditRecord {
            layout: puzzle.layout,
            difficulty: puzzle.difficulty,
            target: config.target_mode,
            operation_profile,
            scramble_len,
            iteration: iteration + 1,
            seed,
            ariadne_unit_len,
            step,
            step_bucket,
            remaining_to_solved,
            ariadne_move: ariadne_move_label.clone(),
            feature,
            candidate_count: candidates.len(),
            first_match_count,
            raw_rank: raw_rank.rank,
            raw_percentile: raw_rank.percentile,
            raw_delta: raw_rank.value,
            first_match_rank: first_match_rank.rank,
            first_match_percentile: first_match_rank.percentile,
            first_match_delta: first_match_rank.value,
            best_delta,
            worst_delta,
        });
    }

    records
}

fn move_delta_metrics(
    puzzle: &Puzzle,
    config: &SolverConfig,
    colors: &[Color],
) -> MoveDeltaMetrics {
    let permutation = permutation_matching_metrics(puzzle, colors);
    MoveDeltaMetrics {
        score: score_state(
            colors,
            puzzle,
            config.target_mode,
            config.region_pair_weight,
            None,
        ),
        ring_entropy: ring_entropy_total(puzzle, colors),
        axis_pair_quality: face_pair_home_matches_key(puzzle, colors)
            .iter()
            .map(|&value| i32::from(value))
            .sum(),
        tape_segment_conflicts: tape_segment_conflicts(puzzle, colors) as i32,
        perm_slot_cost: permutation.slot_cost as i32,
        perm_cycle_cost: permutation.cycle_cost as i32,
        ring_rotation_miss: ring_rotation_miss_total(puzzle, colors) as i32,
    }
}

#[derive(Debug, Clone, Copy)]
struct PermutationMatchingMetrics {
    slot_cost: usize,
    cycle_cost: usize,
}

fn permutation_matching_metrics(puzzle: &Puzzle, colors: &[Color]) -> PermutationMatchingMetrics {
    let mut assignment = vec![0usize; colors.len()];
    let mut slot_cost = 0usize;

    for color in TARGET_COLORS {
        let current = colors
            .iter()
            .enumerate()
            .filter_map(|(index, &value)| (value == color).then_some(index))
            .collect::<Vec<_>>();
        let target = puzzle
            .solved_colors
            .iter()
            .enumerate()
            .filter_map(|(index, &value)| (value == color).then_some(index))
            .collect::<Vec<_>>();

        if current.len() != target.len() {
            return PermutationMatchingMetrics {
                slot_cost: usize::MAX / 4,
                cycle_cost: usize::MAX / 4,
            };
        }
        if current.is_empty() {
            continue;
        }

        let mut costs = vec![vec![0usize; target.len()]; current.len()];
        for (row, &from) in current.iter().enumerate() {
            for (col, &to) in target.iter().enumerate() {
                costs[row][col] = sticker_slot_distance(puzzle, from, to);
            }
        }

        let (cost, matched_targets) = if current.len() <= 12 {
            min_cost_assignment_dp(&costs)
        } else {
            min_cost_assignment_greedy(&costs)
        };
        slot_cost += cost;
        for (row, target_col) in matched_targets.into_iter().enumerate() {
            assignment[current[row]] = target[target_col];
        }
    }

    PermutationMatchingMetrics {
        slot_cost,
        cycle_cost: permutation_cycle_cost(&assignment),
    }
}

fn sticker_slot_distance(puzzle: &Puzzle, from: usize, to: usize) -> usize {
    let a = &puzzle.stickers[from];
    let b = &puzzle.stickers[to];
    a.x.abs_diff(b.x) + a.y.abs_diff(b.y) + a.z.abs_diff(b.z) + usize::from(a.face != b.face) * 3
}

fn min_cost_assignment_dp(costs: &[Vec<usize>]) -> (usize, Vec<usize>) {
    let n = costs.len();
    if n == 0 {
        return (0, Vec::new());
    }
    let states = 1usize << n;
    let inf = usize::MAX / 4;
    let mut dp = vec![inf; states];
    let mut parent = vec![usize::MAX; states];
    dp[0] = 0;

    for mask in 0..states {
        let row = mask.count_ones() as usize;
        if row >= n || dp[mask] == inf {
            continue;
        }
        for col in 0..n {
            if mask & (1usize << col) != 0 {
                continue;
            }
            let next = mask | (1usize << col);
            let next_cost = dp[mask].saturating_add(costs[row][col]);
            if next_cost < dp[next] {
                dp[next] = next_cost;
                parent[next] = col;
            }
        }
    }

    let mut assignment = vec![0usize; n];
    let mut mask = states - 1;
    for row in (0..n).rev() {
        let col = parent[mask];
        assignment[row] = col;
        mask &= !(1usize << col);
    }
    (dp[states - 1], assignment)
}

fn min_cost_assignment_greedy(costs: &[Vec<usize>]) -> (usize, Vec<usize>) {
    let n = costs.len();
    let mut used = vec![false; n];
    let mut assignment = vec![0usize; n];
    let mut total = 0usize;

    for row in 0..n {
        let mut best_col = None;
        let mut best_cost = usize::MAX;
        for col in 0..n {
            if used[col] {
                continue;
            }
            if costs[row][col] < best_cost {
                best_cost = costs[row][col];
                best_col = Some(col);
            }
        }
        let col = best_col.unwrap_or(0);
        used[col] = true;
        assignment[row] = col;
        total = total.saturating_add(best_cost);
    }

    (total, assignment)
}

fn permutation_cycle_cost(permutation: &[usize]) -> usize {
    let mut visited = vec![false; permutation.len()];
    let mut cost = 0usize;

    for start in 0..permutation.len() {
        if visited[start] {
            continue;
        }
        let mut len = 0usize;
        let mut current = start;
        while !visited[current] {
            visited[current] = true;
            len += 1;
            current = permutation[current];
        }
        if len > 1 {
            cost += len - 1;
        }
    }

    cost
}

fn ring_rotation_miss_total(puzzle: &Puzzle, colors: &[Color]) -> usize {
    let mut total = 0usize;
    for coord in tape_coords(puzzle) {
        let Some(cycle) = tape_cycle(puzzle, coord) else {
            continue;
        };
        let current = cycle.iter().map(|&index| colors[index]).collect::<Vec<_>>();
        let target = cycle
            .iter()
            .map(|&index| puzzle.solved_colors[index])
            .collect::<Vec<_>>();
        total += cyclic_hamming_misses(&current, &target);
    }
    total
}

fn ring_entropy_total(puzzle: &Puzzle, colors: &[Color]) -> f64 {
    let mut total = 0.0;
    for coord in tape_coords(puzzle) {
        let Some(cycle) = tape_cycle(puzzle, coord) else {
            continue;
        };
        if cycle.is_empty() {
            continue;
        }
        let mut counts = [0usize; 6];
        for &index in cycle {
            counts[colors[index] as usize] += 1;
        }
        let len = cycle.len() as f64;
        for count in counts {
            if count == 0 {
                continue;
            }
            let p = count as f64 / len;
            total -= p * p.ln();
        }
    }
    total
}

fn tape_segment_conflicts(puzzle: &Puzzle, colors: &[Color]) -> usize {
    let mut conflicts = 0usize;
    for coord in tape_coords(puzzle) {
        let Some(cycle) = tape_cycle(puzzle, coord) else {
            continue;
        };
        if cycle.is_empty() || cycle.len() % 4 != 0 {
            continue;
        }
        let segment_len = cycle.len() / 4;
        for segment in 0..4 {
            let start = segment * segment_len;
            let end = start + segment_len;
            for pair in cycle[start..end].windows(2) {
                if colors[pair[0]] != colors[pair[1]] {
                    conflicts += 1;
                }
            }
        }
    }
    conflicts
}

fn move_delta_rank<F>(
    candidates: &[MoveDeltaCandidate],
    feature_index: usize,
    is_target: F,
) -> MoveDeltaRank
where
    F: Fn(&MoveDeltaCandidate) -> bool,
{
    let Some(target_value) = candidates
        .iter()
        .filter(|candidate| is_target(candidate))
        .map(|candidate| candidate.values[feature_index])
        .max_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal))
    else {
        return MoveDeltaRank {
            rank: 0.0,
            percentile: 100.0,
            value: 0.0,
        };
    };

    let better = candidates
        .iter()
        .filter(|candidate| candidate.values[feature_index] > target_value)
        .count();
    let tied = candidates
        .iter()
        .filter(|candidate| (candidate.values[feature_index] - target_value).abs() <= f64::EPSILON)
        .count();
    let zero_based_rank = better as f64 + tied.saturating_sub(1) as f64 / 2.0;
    let percentile = if candidates.len() <= 1 {
        0.0
    } else {
        zero_based_rank * 100.0 / (candidates.len() - 1) as f64
    };

    MoveDeltaRank {
        rank: zero_based_rank + 1.0,
        percentile,
        value: target_value,
    }
}

fn move_delta_step_bucket(step: usize) -> &'static str {
    match step {
        0..=9 => "0-9",
        10..=19 => "10-19",
        _ => "20+",
    }
}

fn evaluate_beam_direction_survival(
    puzzle: &Puzzle,
    config: &SolverConfig,
    operation_profile: OperationProfile,
    operations: &[Operation],
    pattern_db: Option<&PatternDb>,
    scramble_len: usize,
    iteration: usize,
    seed: u64,
    ariadne_unit_len: usize,
    ariadne_first_move: MoveIndex,
    beam_width: usize,
    audit_depth: usize,
    colors: &[Color],
) -> Vec<BeamDirectionSurvivalRecord> {
    let initial_score = score_state(
        colors,
        puzzle,
        config.target_mode,
        config.region_pair_weight,
        pattern_db,
    );
    let mut beam = vec![DirectionSurvivalBeamEntry {
        colors: colors.to_vec(),
        path_len: 0,
        corridor_prefix: Vec::new(),
        last_move: None,
        first_move: None,
        score: initial_score,
        rank_score: initial_score,
    }];
    let mut records = Vec::with_capacity(audit_depth);
    let mut extinction_layer = None;
    let mut nodes = 0u64;
    let ariadne_first_label = puzzle.move_text(ariadne_first_move);

    for depth_index in 0..audit_depth {
        let depth = depth_index + 1;
        let mut candidates = Vec::new();
        let mut layer_seen = HashSet::new();

        for entry in &beam {
            for operation in operations {
                if operation.is_raw
                    && entry
                        .last_move
                        .is_some_and(|last| puzzle.moves[operation.moves[0]].inverse == last)
                {
                    continue;
                }

                let mut next_colors = entry.colors.clone();
                for &move_index in &operation.moves {
                    puzzle.apply_move(&mut next_colors, move_index);
                }
                nodes += 1;

                let key = state_key(&next_colors);
                if !layer_seen.insert(key) {
                    continue;
                }

                let path_len = entry.path_len + operation.path.len();
                let first_move = entry.first_move.or_else(|| operation.path.first().copied());
                let corridor_prefix = extend_corridor_prefix(
                    &entry.corridor_prefix,
                    &operation.path,
                    config.corridor_prefix_len,
                );
                let score = score_state(
                    &next_colors,
                    puzzle,
                    config.target_mode,
                    config.region_pair_weight,
                    pattern_db,
                ) + path_len as i32 * config.path_penalty;
                let rank_score = beam_rank_score_with_len(
                    puzzle,
                    &next_colors,
                    path_len,
                    score,
                    pattern_db,
                    config,
                );

                candidates.push(DirectionSurvivalBeamEntry {
                    colors: next_colors,
                    path_len,
                    corridor_prefix,
                    last_move: Some(operation.last_move),
                    first_move,
                    score,
                    rank_score,
                });
            }
        }

        if candidates.is_empty() {
            break;
        }

        candidates.sort_unstable_by(|left, right| {
            left.rank_score
                .cmp(&right.rank_score)
                .then_with(|| left.path_len.cmp(&right.path_len))
                .then_with(|| left.score.cmp(&right.score))
        });
        beam = select_corridor_diverse_by_path(
            candidates,
            beam_width,
            config.corridor_diversity_enabled,
            config.corridor_prefix_len,
            config.corridor_quota_percent,
            |entry| &entry.corridor_prefix,
        );

        let mut direction_entries = [0usize; 32];
        for entry in &beam {
            if let Some(first_move) = entry.first_move {
                if let Some(slot) = direction_entries.get_mut(first_move) {
                    *slot += 1;
                }
            }
        }
        let direction_count = direction_entries.iter().filter(|&&count| count > 0).count();
        let target_direction_entries = direction_entries
            .get(ariadne_first_move)
            .copied()
            .unwrap_or_default();
        let max_direction_entries = direction_entries.iter().copied().max().unwrap_or_default();
        let survival = target_direction_entries > 0;
        if !survival {
            extinction_layer.get_or_insert(depth);
        }

        records.push(BeamDirectionSurvivalRecord {
            layout: puzzle.layout,
            difficulty: puzzle.difficulty,
            target: config.target_mode,
            operation_profile,
            scramble_len,
            iteration: iteration + 1,
            seed,
            ariadne_unit_len,
            ariadne_first_move: ariadne_first_label.clone(),
            beam_width,
            depth,
            survival,
            extinction_layer,
            direction_count,
            target_direction_entries,
            max_direction_entries,
            beam_size: beam.len(),
            nodes,
        });
    }

    records
}

fn evaluate_beam_prefix_survival(
    puzzle: &Puzzle,
    config: &SolverConfig,
    operation_profile: OperationProfile,
    operations: &[Operation],
    pattern_db: Option<&PatternDb>,
    scramble_len: usize,
    iteration: usize,
    seed: u64,
    ariadne_solution: &[MoveIndex],
    beam_width: usize,
    audit_depth: usize,
    colors: &[Color],
) -> Vec<BeamPrefixSurvivalRecord> {
    let initial_score = score_state(
        colors,
        puzzle,
        config.target_mode,
        config.region_pair_weight,
        pattern_db,
    );
    let mut beam = vec![PrefixSurvivalBeamEntry {
        colors: colors.to_vec(),
        path_len: 0,
        corridor_prefix: Vec::new(),
        last_move: None,
        matching_prefix_len: 0,
        score: initial_score,
        rank_score: initial_score,
    }];
    let max_prefix_to_check = audit_depth.min(ariadne_solution.len());
    let prefix_state_keys =
        ariadne_prefix_state_keys(puzzle, colors, ariadne_solution, max_prefix_to_check);
    let mut records = Vec::with_capacity(audit_depth);
    let mut nodes = 0u64;

    for depth_index in 0..audit_depth {
        let depth = depth_index + 1;
        let mut candidates = Vec::new();
        let mut layer_seen = HashSet::new();

        for entry in &beam {
            for operation in operations {
                if operation.is_raw
                    && entry
                        .last_move
                        .is_some_and(|last| puzzle.moves[operation.moves[0]].inverse == last)
                {
                    continue;
                }

                let mut next_colors = entry.colors.clone();
                for &move_index in &operation.moves {
                    puzzle.apply_move(&mut next_colors, move_index);
                }
                nodes += 1;

                let key = state_key(&next_colors);
                if !layer_seen.insert(key) {
                    continue;
                }

                let path_len = entry.path_len + operation.path.len();
                let corridor_prefix = extend_corridor_prefix(
                    &entry.corridor_prefix,
                    &operation.path,
                    config.corridor_prefix_len,
                );
                let matching_prefix_len = extend_matching_prefix(
                    entry.matching_prefix_len,
                    entry.path_len,
                    &operation.path,
                    ariadne_solution,
                );
                let score = score_state(
                    &next_colors,
                    puzzle,
                    config.target_mode,
                    config.region_pair_weight,
                    pattern_db,
                ) + path_len as i32 * config.path_penalty;
                let rank_score = beam_rank_score_with_len(
                    puzzle,
                    &next_colors,
                    path_len,
                    score,
                    pattern_db,
                    config,
                );

                candidates.push(PrefixSurvivalBeamEntry {
                    colors: next_colors,
                    path_len,
                    corridor_prefix,
                    last_move: Some(operation.last_move),
                    matching_prefix_len,
                    score,
                    rank_score,
                });
            }
        }

        if candidates.is_empty() {
            break;
        }

        candidates.sort_unstable_by(|left, right| {
            left.rank_score
                .cmp(&right.rank_score)
                .then_with(|| left.path_len.cmp(&right.path_len))
                .then_with(|| left.score.cmp(&right.score))
        });
        beam = select_corridor_diverse_by_path(
            candidates,
            beam_width,
            config.corridor_diversity_enabled,
            config.corridor_prefix_len,
            config.corridor_quota_percent,
            |entry| &entry.corridor_prefix,
        );

        let target_prefix_len = depth.min(ariadne_solution.len());
        let matching_states_count = beam
            .iter()
            .filter(|entry| entry.matching_prefix_len >= target_prefix_len)
            .count();
        let strict_path_state_count = beam
            .iter()
            .filter(|entry| {
                entry.path_len == target_prefix_len
                    && entry.matching_prefix_len >= target_prefix_len
            })
            .count();
        let max_matching_prefix = beam
            .iter()
            .map(|entry| entry.matching_prefix_len)
            .max()
            .unwrap_or_default();
        let max_prefix_entries = beam
            .iter()
            .filter(|entry| entry.matching_prefix_len == max_matching_prefix)
            .count();
        let mean_matching_prefix = if beam.is_empty() {
            0.0
        } else {
            beam.iter()
                .map(|entry| entry.matching_prefix_len as f64)
                .sum::<f64>()
                / beam.len() as f64
        };
        let beam_state_keys = beam
            .iter()
            .map(|entry| state_key(&entry.colors))
            .collect::<HashSet<_>>();
        let prefix_state_alive = prefix_state_keys
            .get(target_prefix_len)
            .is_some_and(|key| beam_state_keys.contains(key));

        records.push(BeamPrefixSurvivalRecord {
            layout: puzzle.layout,
            difficulty: puzzle.difficulty,
            target: config.target_mode,
            operation_profile,
            scramble_len,
            iteration: iteration + 1,
            seed,
            ariadne_unit_len: ariadne_solution.len(),
            beam_width,
            depth,
            target_prefix_len,
            path_prefix_alive: matching_states_count > 0,
            prefix_state_alive,
            max_matching_prefix,
            matching_states_count,
            strict_path_state_count,
            max_prefix_entries,
            mean_matching_prefix,
            beam_size: beam.len(),
            nodes,
        });
    }

    records
}

fn extend_corridor_prefix(
    current: &[MoveIndex],
    appended: &[MoveIndex],
    prefix_len: usize,
) -> Vec<MoveIndex> {
    if current.len() >= prefix_len {
        return current.to_vec();
    }

    let mut out = Vec::with_capacity(prefix_len);
    out.extend_from_slice(current);
    for &move_index in appended {
        if out.len() >= prefix_len {
            break;
        }
        out.push(move_index);
    }
    out
}

fn extend_matching_prefix(
    current_match: usize,
    current_path_len: usize,
    appended: &[MoveIndex],
    ariadne_solution: &[MoveIndex],
) -> usize {
    if current_match != current_path_len {
        return current_match;
    }

    let mut matched = current_match;
    for &move_index in appended {
        if ariadne_solution.get(matched).copied() == Some(move_index) {
            matched += 1;
        } else {
            break;
        }
    }
    matched
}

fn ariadne_prefix_state_keys(
    puzzle: &Puzzle,
    colors: &[Color],
    ariadne_solution: &[MoveIndex],
    max_prefix: usize,
) -> Vec<Key> {
    let mut out = Vec::with_capacity(max_prefix + 1);
    let mut current = colors.to_vec();
    out.push(state_key(&current));
    for &move_index in ariadne_solution.iter().take(max_prefix) {
        puzzle.apply_move(&mut current, move_index);
        out.push(state_key(&current));
    }
    out
}

fn evaluate_backward_midpoint(
    puzzle: &Puzzle,
    config: &SolverConfig,
    operation_profile: OperationProfile,
    operations: &[Operation],
    pattern_db: Option<&PatternDb>,
    scramble_len: usize,
    iteration: usize,
    seed: u64,
    ariadne_solution: &[MoveIndex],
    scrambled_colors: &[Color],
    target_states: &[Vec<Color>],
    beam_width: usize,
    audit_depth: usize,
    midpoint_step: usize,
) -> BackwardMidpointAuditRecord {
    let started = Instant::now();
    let midpoint_step = midpoint_step.min(ariadne_solution.len());
    let mut midpoint_colors = scrambled_colors.to_vec();
    for &move_index in &ariadne_solution[..midpoint_step] {
        puzzle.apply_move(&mut midpoint_colors, move_index);
    }
    let midpoint_key = state_key(&midpoint_colors);
    let target_score = score_state(
        &midpoint_colors,
        puzzle,
        config.target_mode,
        config.region_pair_weight,
        pattern_db,
    );

    let mut beam = Vec::new();
    let mut initial_seen = HashSet::new();
    for target in target_states {
        let key = state_key(target);
        if !initial_seen.insert(key) {
            continue;
        }
        let score = score_state(
            target,
            puzzle,
            config.target_mode,
            config.region_pair_weight,
            pattern_db,
        );
        let rank_score = beam_rank_score_with_len(puzzle, target, 0, score, pattern_db, config);
        beam.push(BackwardMidpointBeamEntry {
            colors: target.clone(),
            key,
            path_len: 0,
            last_move: None,
            score,
            rank_score,
        });
    }
    beam.sort_unstable_by(|left, right| {
        left.rank_score
            .cmp(&right.rank_score)
            .then_with(|| left.path_len.cmp(&right.path_len))
            .then_with(|| left.score.cmp(&right.score))
    });
    beam.truncate(beam_width);

    let mut nodes = 0u64;
    let mut limits = Limits::new(config.max_nodes, config.time_limit_ms);
    let mut candidate_hit = beam.iter().any(|entry| entry.key == midpoint_key);
    let mut selected_hit = candidate_hit;
    let mut first_candidate_layer = candidate_hit.then_some(0);
    let mut first_selected_layer = selected_hit.then_some(0);
    let mut best_candidate_layer = candidate_hit.then_some(0);
    let mut best_candidate_rank = candidate_hit.then_some(1usize);
    let mut best_candidate_count = candidate_hit.then_some(beam.len());
    let mut best_candidate_percentile = candidate_hit.then_some(if beam.is_empty() {
        0.0
    } else {
        100.0 / beam.len() as f64
    });

    'layers: for layer in 1..=audit_depth {
        let mut candidates = Vec::new();
        let mut layer_seen = HashSet::new();

        for entry in &beam {
            for operation in operations {
                if operation.is_raw
                    && entry
                        .last_move
                        .is_some_and(|last| puzzle.moves[operation.moves[0]].inverse == last)
                {
                    continue;
                }

                let mut next_colors = entry.colors.clone();
                for &move_index in &operation.moves {
                    puzzle.apply_move(&mut next_colors, move_index);
                }
                nodes += 1;
                if limits.exceeded(nodes) {
                    break 'layers;
                }

                let key = state_key(&next_colors);
                if !layer_seen.insert(key) {
                    continue;
                }

                let path_len = entry.path_len + operation.path.len();
                let score = score_state(
                    &next_colors,
                    puzzle,
                    config.target_mode,
                    config.region_pair_weight,
                    pattern_db,
                ) + path_len as i32 * config.path_penalty;
                let rank_score = beam_rank_score_with_len(
                    puzzle,
                    &next_colors,
                    path_len,
                    score,
                    pattern_db,
                    config,
                );
                candidates.push(BackwardMidpointBeamEntry {
                    colors: next_colors,
                    key,
                    path_len,
                    last_move: Some(operation.last_move),
                    score,
                    rank_score,
                });
            }
        }

        if candidates.is_empty() {
            beam.clear();
            break;
        }

        candidates.sort_unstable_by(|left, right| {
            left.rank_score
                .cmp(&right.rank_score)
                .then_with(|| left.path_len.cmp(&right.path_len))
                .then_with(|| left.score.cmp(&right.score))
        });

        if let Some(position) = candidates
            .iter()
            .position(|entry| entry.key == midpoint_key)
        {
            let rank = position + 1;
            let percentile = rank as f64 * 100.0 / candidates.len() as f64;
            candidate_hit = true;
            first_candidate_layer.get_or_insert(layer);
            if best_candidate_rank
                .map(|best_rank| rank < best_rank)
                .unwrap_or(true)
            {
                best_candidate_layer = Some(layer);
                best_candidate_rank = Some(rank);
                best_candidate_count = Some(candidates.len());
                best_candidate_percentile = Some(percentile);
            }
        }

        candidates.truncate(beam_width);
        beam = candidates;

        if beam.iter().any(|entry| entry.key == midpoint_key) {
            selected_hit = true;
            first_selected_layer.get_or_insert(layer);
        }
    }

    let final_frontier_hit = beam.iter().any(|entry| entry.key == midpoint_key);
    let final_best_score = beam.iter().map(|entry| entry.score).min();
    let final_worst_score = beam.iter().map(|entry| entry.score).max();

    BackwardMidpointAuditRecord {
        layout: puzzle.layout,
        difficulty: puzzle.difficulty,
        target: config.target_mode,
        operation_profile,
        scramble_len,
        iteration: iteration + 1,
        seed,
        ariadne_unit_len: ariadne_solution.len(),
        midpoint_step,
        beam_width,
        depth: audit_depth,
        target_states: target_states.len(),
        operation_count: operations.len(),
        candidate_hit,
        selected_hit,
        final_frontier_hit,
        first_candidate_layer,
        first_selected_layer,
        best_candidate_layer,
        best_candidate_rank,
        best_candidate_count,
        best_candidate_percentile,
        final_frontier_size: beam.len(),
        target_score,
        final_best_score,
        final_worst_score,
        nodes,
        elapsed_ms: started.elapsed().as_millis(),
    }
}

fn ratio(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

fn run_phase_lab(options: BenchOptions) -> io::Result<()> {
    fs::create_dir_all(&options.out_dir)?;

    let stamp = unix_stamp();
    let report_prefix = if options.e_classic_cascade {
        "e_classic_cascade"
    } else {
        "phase_lab"
    };
    let csv_path = options.out_dir.join(format!("{report_prefix}_{stamp}.csv"));
    let md_path = options.out_dir.join(format!("{report_prefix}_{stamp}.md"));
    let mut records = Vec::new();

    println!(
        "{} target={} scramble_profile={} phase_kinds={} layouts={} difficulties={} scrambles={:?} iterations={} iteration_start={} seed={}",
        report_prefix,
        options.solver.target_mode.label(),
        options.scramble_profile.label(),
        join_phase_kinds(&options.phase_kinds),
        join_display(&options.layouts),
        join_display(&options.difficulties),
        options.scramble_lengths,
        iteration_label(&options),
        options.iteration_start + 1,
        options.seed
    );

    for &layout in &options.layouts {
        for &difficulty in &options.difficulties {
            let puzzle = Puzzle::new(layout, difficulty);
            let artifacts = prepare_solver(&puzzle, &options.solver)?;
            let axis_ring_rescue_available = options.e_classic_cascade
                && options.axis_ring_rescue_enabled
                && layout == LayoutId::E
                && difficulty == Difficulty::Classic;
            let mut axis_ring_rescue_artifacts: Option<AxisRingRescueArtifacts> = None;
            let mut phase_specs = if options.e_classic_cascade {
                let mut specs =
                    build_phase_specs(&puzzle, &[PhaseKind::AllOppositeAndroidNearPairs], 2);
                specs.extend(build_phase_specs(
                    &puzzle,
                    &[PhaseKind::ProtectedCornerArms],
                    0,
                ));
                specs
            } else {
                build_phase_specs(&puzzle, &options.phase_kinds, options.phase_near_misses)
            };
            if !options.phase_spec_filters.is_empty() {
                phase_specs.retain(|spec| {
                    let label = spec.label.to_ascii_lowercase();
                    options
                        .phase_spec_filters
                        .iter()
                        .any(|filter| label.contains(filter))
                });
            }
            let phase_operation_sets = build_phase_operation_sets(&puzzle, &options);
            let cascade_near_options = options
                .e_classic_cascade
                .then(|| e_classic_near_pair_options(&options));
            let cascade_corner_options = options
                .e_classic_cascade
                .then(|| e_classic_corner_options(&options));
            let cascade_near_operation_sets = cascade_near_options
                .as_ref()
                .map(|stage_options| build_phase_operation_sets(&puzzle, stage_options));
            let cascade_corner_operation_sets = cascade_corner_options
                .as_ref()
                .map(|stage_options| build_phase_operation_sets(&puzzle, stage_options));
            let operation_summary = if options.e_classic_cascade {
                format!(
                    "near=[{}],corner=[{}]",
                    operation_sets_summary(
                        cascade_near_operation_sets
                            .as_deref()
                            .expect("cascade near operation sets")
                    ),
                    operation_sets_summary(
                        cascade_corner_operation_sets
                            .as_deref()
                            .expect("cascade corner operation sets")
                    )
                )
            } else {
                operation_sets_summary(&phase_operation_sets)
            };

            println!(
                "prepared {layout}-{difficulty}: specs={} phase_ops=[{}] variants={} axis_ring_rescue={} build_nodes={} build_time={}ms [{}]",
                phase_specs.len(),
                operation_summary,
                artifacts.variants.len(),
                if axis_ring_rescue_available {
                    format!(
                        "lazy,position={},expand={},threshold={},corner_skip<={},tier={},{},{}",
                        options.axis_ring_rescue_position.label(),
                        options.axis_ring_rescue_expand_depth,
                        options.axis_ring_rescue_threshold,
                        options.axis_ring_rescue_corner_skip_threshold,
                        options.axis_ring_rescue_tier.max_depth,
                        options.axis_ring_rescue_tier.width,
                        options.axis_ring_rescue_tier.restarts
                    )
                } else {
                    "off".to_string()
                },
                artifacts.build_nodes,
                artifacts.build_ms,
                artifact_summary(&artifacts)
            );

            for &scramble_len in &options.scramble_lengths {
                for iteration in iteration_values(&options) {
                    let run_seed =
                        derive_seed(options.seed, layout, difficulty, scramble_len, iteration);
                    let scramble = generate_scramble(
                        &puzzle,
                        scramble_len,
                        run_seed,
                        options.scramble_profile,
                        options.avoid_same_tape,
                    );
                    let ariadne_solution_len = ariadne_solution_len(&puzzle, &scramble);
                    let mut scrambled_colors = puzzle.solved_colors.clone();
                    puzzle.apply_moves(&mut scrambled_colors, &scramble);

                    let direct = if options.phase_skip_direct {
                        SolveResult {
                            found: false,
                            method: SolveMethod::None,
                            target_used: None,
                            operation_profile_used: None,
                            reason: "direct:skipped".to_string(),
                            raw_moves: Vec::new(),
                            optimized_moves: Vec::new(),
                            nodes: 0,
                            elapsed_ms: 0,
                            first_table_hit: None,
                        }
                    } else {
                        solve_puzzle(&puzzle, &scrambled_colors, &options.solver, &artifacts)
                    };
                    let direct_opt_len = direct.optimized_moves.len();
                    let skip_phase = direct.found
                        && options.phase_direct_threshold > 0
                        && direct_opt_len < options.phase_direct_threshold;

                    let mut cascade_phase_found = false;
                    let mut cascade_found = direct.found;
                    let mut cascade_selected_stage = if direct.found { "direct" } else { "none" };
                    let mut cascade_raw_len = if direct.found {
                        direct.raw_moves.len()
                    } else {
                        0
                    };
                    let mut cascade_opt_len = if direct.found { direct_opt_len } else { 0 };
                    let mut cascade_prefix_len = 0usize;
                    let mut cascade_suffix_len = 0usize;
                    let mut cascade_phase_elapsed_ms = 0u128;
                    let mut cascade_suffix_elapsed_ms = 0u128;
                    let mut cascade_nodes = direct.nodes;
                    let mut cascade_prefixes_available = 0usize;
                    let mut cascade_prefixes_tested = 0usize;
                    let mut cascade_candidate_min_prefix_len = 0usize;
                    let mut cascade_candidate_prefix_lens = String::new();
                    let mut cascade_candidate_signatures = String::new();
                    let mut axis_ring_rescue_before_corner_attempted = false;
                    let mut cascade_reasons = vec![format!(
                        "direct:{}:{}",
                        if direct.found { "found" } else { "not_found" },
                        direct.reason
                    )];

                    for (spec_index, spec) in phase_specs.iter().enumerate() {
                        let corner_stage = matches!(
                            spec.kind,
                            PhaseKind::ProtectedCornerArms | PhaseKind::ProtectedCornerBlock
                        );
                        let phase_options = if options.e_classic_cascade {
                            if corner_stage {
                                cascade_corner_options
                                    .as_ref()
                                    .expect("cascade corner options")
                            } else {
                                cascade_near_options.as_ref().expect("cascade near options")
                            }
                        } else {
                            &options
                        };
                        let stage_operation_sets = if options.e_classic_cascade {
                            if corner_stage {
                                cascade_corner_operation_sets
                                    .as_deref()
                                    .expect("cascade corner operation sets")
                            } else {
                                cascade_near_operation_sets
                                    .as_deref()
                                    .expect("cascade near operation sets")
                            }
                        } else {
                            phase_operation_sets.as_slice()
                        };
                        let pool_corner_specs = phase_options.phase_corner_pool_specs
                            && matches!(
                                spec.kind,
                                PhaseKind::ProtectedCornerArms | PhaseKind::ProtectedCornerBlock
                            );
                        if pool_corner_specs
                            && phase_specs[..spec_index]
                                .iter()
                                .any(|previous| previous.kind == spec.kind)
                        {
                            continue;
                        }
                        let source_specs = if pool_corner_specs {
                            phase_specs
                                .iter()
                                .filter(|source| source.kind == spec.kind)
                                .collect::<Vec<_>>()
                        } else {
                            vec![spec]
                        };
                        let phase_spec_label = if pool_corner_specs {
                            format!(
                                "{}:pooled-corners={}:miss<={}",
                                spec.kind.label(),
                                source_specs.len(),
                                spec.near_misses_per_face
                            )
                        } else {
                            spec.label.clone()
                        };
                        if options.e_classic_cascade
                            && corner_stage
                            && axis_ring_rescue_available
                            && options.axis_ring_rescue_position.runs_before_corner()
                            && !axis_ring_rescue_before_corner_attempted
                            && should_run_axis_ring_rescue(
                                &options,
                                &puzzle,
                                cascade_found,
                                cascade_opt_len,
                            )
                        {
                            let cascade_found_before_axis = cascade_found;
                            let cascade_opt_before_axis = cascade_opt_len;
                            let axis_record = run_axis_ring_rescue_record(
                                &puzzle,
                                &options,
                                &mut axis_ring_rescue_artifacts,
                                &artifacts,
                                layout,
                                difficulty,
                                scramble_len,
                                iteration,
                                run_seed,
                                ariadne_solution_len,
                                &direct,
                                cascade_found,
                                cascade_opt_len,
                                &scrambled_colors,
                            )?;
                            axis_ring_rescue_before_corner_attempted = true;
                            cascade_phase_found |= axis_record.phase_found;
                            cascade_suffix_elapsed_ms += axis_record.total_elapsed_ms;
                            cascade_nodes += axis_record.nodes;
                            if axis_record.total_found
                                && (!cascade_found || axis_record.total_opt_len < cascade_opt_len)
                            {
                                cascade_found = true;
                                cascade_selected_stage = "axis-ring";
                                cascade_raw_len = axis_record.total_raw_len;
                                cascade_opt_len = axis_record.total_opt_len;
                                cascade_prefix_len = axis_record.prefix_len;
                                cascade_suffix_len = axis_record.suffix_len;
                                cascade_prefixes_available = axis_record.prefixes_available;
                                cascade_prefixes_tested = axis_record.prefixes_tested;
                                cascade_candidate_min_prefix_len =
                                    axis_record.candidate_min_prefix_len;
                                cascade_candidate_prefix_lens =
                                    axis_record.candidate_prefix_lens.clone();
                                cascade_candidate_signatures =
                                    axis_record.candidate_signatures.clone();
                            }
                            cascade_reasons.push(format!(
                                "axis-ring-before-corner:{}:{}",
                                if axis_record.total_found {
                                    "found"
                                } else {
                                    "not_found"
                                },
                                axis_record.reason
                            ));
                            println!(
                                "{layout}-{difficulty} scramble={scramble_len} iter={} axis_ring_before_corner found={} opt={} cascade_before={} time={}ms nodes={} reason={}",
                                iteration + 1,
                                axis_record.total_found,
                                if axis_record.total_found {
                                    axis_record.total_opt_len.to_string()
                                } else {
                                    "-".to_string()
                                },
                                if cascade_found_before_axis {
                                    cascade_opt_before_axis.to_string()
                                } else {
                                    "-".to_string()
                                },
                                axis_record.total_elapsed_ms,
                                axis_record.nodes,
                                axis_record.reason
                            );
                            records.push(axis_record);
                        }
                        let corner_skipped_after_axis_ring = options.e_classic_cascade
                            && corner_stage
                            && axis_ring_rescue_before_corner_attempted
                            && cascade_found
                            && should_skip_corner_after_axis_ring(&options, cascade_opt_len);
                        if options.e_classic_cascade
                            && corner_stage
                            && (corner_skipped_after_axis_ring
                                || !should_run_e_classic_corner_rescue(
                                    scramble_len,
                                    direct.found,
                                    direct_opt_len,
                                    cascade_found,
                                    cascade_opt_len,
                                ))
                        {
                            let reason = if corner_skipped_after_axis_ring {
                                "cascade:corner_skipped_after_axis_ring_threshold"
                            } else if scramble_len < E_CLASSIC_CORNER_RESCUE_MIN_SCRAMBLE {
                                "cascade:corner_disabled_scramble_lt_min"
                            } else if cascade_found
                                && cascade_opt_len <= E_CLASSIC_CORNER_QUALITY_THRESHOLD
                            {
                                "cascade:corner_not_needed_current_within_threshold"
                            } else {
                                "cascade:corner_not_needed_direct_within_threshold"
                            };
                            cascade_reasons.push(reason.to_string());
                            let record = PhaseLabRecord {
                                layout,
                                difficulty,
                                scramble_len,
                                iteration: iteration + 1,
                                seed: run_seed,
                                ariadne_solution_len,
                                phase_kind: spec.kind.label().to_string(),
                                phase_spec: phase_spec_label,
                                direct_found: direct.found,
                                direct_opt_len,
                                direct_elapsed_ms: direct.elapsed_ms,
                                phase_found: false,
                                suffix_found: false,
                                total_found: false,
                                prefix_len: 0,
                                suffix_len: 0,
                                total_raw_len: 0,
                                total_opt_len: 0,
                                delta_opt_vs_direct: 0,
                                phase_elapsed_ms: 0,
                                suffix_elapsed_ms: 0,
                                total_elapsed_ms: 0,
                                nodes: 0,
                                prefixes_available: 0,
                                prefixes_tested: 0,
                                candidate_min_prefix_len: 0,
                                candidate_prefix_lens: String::new(),
                                candidate_signatures: String::new(),
                                reason: reason.to_string(),
                            };
                            println!(
                                "{layout}-{difficulty} scramble={scramble_len} iter={} spec={} skipped={reason}",
                                iteration + 1,
                                record.phase_spec
                            );
                            records.push(record);
                            continue;
                        }
                        if skip_phase {
                            let record = PhaseLabRecord {
                                layout,
                                difficulty,
                                scramble_len,
                                iteration: iteration + 1,
                                seed: run_seed,
                                ariadne_solution_len,
                                phase_kind: spec.kind.label().to_string(),
                                phase_spec: phase_spec_label,
                                direct_found: true,
                                direct_opt_len,
                                direct_elapsed_ms: direct.elapsed_ms,
                                phase_found: false,
                                suffix_found: false,
                                total_found: false,
                                prefix_len: 0,
                                suffix_len: 0,
                                total_raw_len: 0,
                                total_opt_len: 0,
                                delta_opt_vs_direct: 0,
                                phase_elapsed_ms: 0,
                                suffix_elapsed_ms: 0,
                                total_elapsed_ms: 0,
                                nodes: 0,
                                prefixes_available: 0,
                                prefixes_tested: 0,
                                candidate_min_prefix_len: 0,
                                candidate_prefix_lens: String::new(),
                                candidate_signatures: String::new(),
                                reason: format!(
                                    "phase:skipped_direct_below_threshold={}",
                                    options.phase_direct_threshold
                                ),
                            };
                            println!(
                                "{layout}-{difficulty} scramble={scramble_len} iter={} spec={} direct={} phase_skipped=threshold<{}",
                                iteration + 1,
                                record.phase_spec,
                                record.direct_opt_len,
                                options.phase_direct_threshold
                            );
                            records.push(record);
                            continue;
                        }

                        let phase_started = Instant::now();
                        let mut phase_nodes = 0u64;
                        let mut phase_reasons = BTreeMap::new();
                        let mut prefixes = Vec::new();
                        let mut seen_prefixes = HashSet::new();
                        let probe_prefixes = phase_options
                            .phase_probe_prefixes
                            .max(phase_options.phase_prefixes);

                        for operation_set in stage_operation_sets {
                            for source_spec in &source_specs {
                                let mut profile_nodes = 0u64;
                                let mut phase_limits = Limits::new(
                                    phase_options.solver.max_nodes,
                                    phase_options.phase_time_limit_ms,
                                );
                                let profile_prefixes = if matches!(
                                    source_spec.kind,
                                    PhaseKind::ProtectedCornerArms
                                        | PhaseKind::ProtectedCornerBlock
                                ) {
                                    find_protected_corner_prefixes(
                                        &puzzle,
                                        &scrambled_colors,
                                        source_spec,
                                        &operation_set.operations,
                                        phase_options.phase_tier,
                                        probe_prefixes,
                                        phase_options.solver.path_penalty,
                                        phase_options.phase_corner_shielded,
                                        phase_options.phase_corner_shielded_body_depth,
                                        phase_options.phase_corner_seed_branches,
                                        phase_options.phase_corner_arm_branches,
                                        &mut profile_nodes,
                                        &mut phase_limits,
                                    )
                                } else {
                                    find_phase_prefixes(
                                        &puzzle,
                                        &scrambled_colors,
                                        source_spec,
                                        &operation_set.operations,
                                        phase_options.phase_tier,
                                        probe_prefixes,
                                        phase_options.solver.path_penalty,
                                        &mut profile_nodes,
                                        &mut phase_limits,
                                    )
                                };
                                phase_nodes += profile_nodes;

                                if profile_prefixes.is_empty() {
                                    let reason = phase_limits
                                        .stop_reason
                                        .clone()
                                        .unwrap_or_else(|| "not_found".to_string());
                                    *phase_reasons
                                        .entry(format!(
                                            "phase:{}:{reason}",
                                            operation_set.profile.label()
                                        ))
                                        .or_insert(0) += 1;
                                    continue;
                                }

                                for prefix in profile_prefixes {
                                    record_prefix(
                                        &mut prefixes,
                                        &mut seen_prefixes,
                                        prefix,
                                        usize::MAX,
                                    );
                                }
                            }
                        }
                        if pool_corner_specs {
                            dedupe_phase_prefix_states(&puzzle, &scrambled_colors, &mut prefixes);
                        }
                        if let Some(max_over_min) = phase_options.phase_prefix_max_over_min {
                            if let Some(min_len) = prefixes.iter().map(Vec::len).min() {
                                let max_len = min_len.saturating_add(max_over_min);
                                prefixes.retain(|prefix| prefix.len() <= max_len);
                            }
                        }
                        phase_nodes += rank_phase_prefixes(
                            &puzzle,
                            &scrambled_colors,
                            spec,
                            phase_options,
                            &mut prefixes,
                            &artifacts,
                            artifacts
                                .variants
                                .first()
                                .and_then(|variant| variant.pattern_db.as_ref()),
                        );
                        let prefixes_available = prefixes.len();
                        let candidate_min_prefix_len =
                            prefixes.iter().map(Vec::len).min().unwrap_or(0);
                        let candidate_prefix_lens = prefixes
                            .iter()
                            .map(|prefix| prefix.len().to_string())
                            .collect::<Vec<_>>()
                            .join(",");
                        let candidate_signatures = prefixes
                            .iter()
                            .map(|prefix| {
                                let mut intermediate_colors = scrambled_colors.clone();
                                puzzle.apply_moves(&mut intermediate_colors, prefix);
                                let global_score = score_state(
                                    &intermediate_colors,
                                    &puzzle,
                                    acceptance_target(options.solver.target_mode),
                                    phase_options.solver.region_pair_weight,
                                    None,
                                );
                                let phase_score = phase_score(
                                    &puzzle,
                                    &intermediate_colors,
                                    spec,
                                    prefix.len(),
                                    phase_options.solver.path_penalty,
                                );
                                let lookahead_score = phase_prefix_lookahead_score(
                                    &puzzle,
                                    &intermediate_colors,
                                    acceptance_target(options.solver.target_mode),
                                    phase_options.solver.region_pair_weight,
                                    None,
                                    phase_options.phase_rank_lookahead_depth,
                                    phase_options.phase_rank_lookahead_width,
                                );
                                format!(
                                    "{:08x}:{}:{}:{}:{}",
                                    hash_colors(&intermediate_colors),
                                    prefix.len(),
                                    global_score,
                                    phase_score,
                                    lookahead_score
                                )
                            })
                            .collect::<Vec<_>>()
                            .join(",");
                        let selected_prefix_offset =
                            phase_options.phase_prefix_offset.min(prefixes.len());
                        prefixes.drain(..selected_prefix_offset);
                        prefixes.truncate(phase_options.phase_prefixes);
                        let phase_elapsed_ms = phase_started.elapsed().as_millis();

                        let mut best: Option<SolveResult> = None;
                        let mut suffix_nodes = 0u64;
                        let mut suffix_elapsed_ms = 0u128;
                        let mut reasons = BTreeMap::new();
                        let mut prefixes_tested = 0usize;
                        let mut stopped_early = false;
                        let mut stopped_after_primary_corner_batch = false;

                        if prefixes.is_empty() {
                            *reasons
                                .entry(if phase_reasons.is_empty() {
                                    "phase:not_found".to_string()
                                } else {
                                    join_counts(&phase_reasons)
                                })
                                .or_insert(0) += 1;
                        }

                        for (prefix_index, prefix) in prefixes.iter().enumerate() {
                            prefixes_tested += 1;
                            let mut intermediate_colors = scrambled_colors.clone();
                            puzzle.apply_moves(&mut intermediate_colors, prefix);
                            let suffix = solve_suffix_from_prefix(
                                &puzzle,
                                &scrambled_colors,
                                &intermediate_colors,
                                prefix,
                                prefix_index + selected_prefix_offset,
                                &phase_options.solver,
                                &artifacts,
                                phase_options.phase_suffix_time_limit_ms,
                                phase_options.phase_suffix_hard_rescue,
                            );
                            suffix_nodes += suffix.nodes;
                            suffix_elapsed_ms += suffix.elapsed_ms;
                            if suffix.found {
                                let reached_stop_gain = direct.found
                                    && phase_options.phase_stop_after_gain.is_some_and(|gain| {
                                        suffix.optimized_moves.len().saturating_add(gain)
                                            <= direct_opt_len
                                    });
                                keep_best_result(&mut best, suffix);
                                if reached_stop_gain {
                                    stopped_early = true;
                                    break;
                                }
                            } else {
                                *reasons.entry(suffix.reason).or_insert(0) += 1;
                            }
                            if options.e_classic_cascade
                                && corner_stage
                                && prefixes_tested == E_CLASSIC_CORNER_PRIMARY_PREFIXES
                                && best.is_some()
                            {
                                stopped_after_primary_corner_batch = true;
                                break;
                            }
                        }

                        let (
                            total_found,
                            prefix_len,
                            suffix_len,
                            total_raw_len,
                            total_opt_len,
                            reason,
                        ) = if let Some(mut best) = best {
                            if stopped_early {
                                best.reason.push_str(&format!(
                                    ":early_stop_gain={}",
                                    phase_options.phase_stop_after_gain.unwrap_or(0)
                                ));
                            }
                            if stopped_after_primary_corner_batch {
                                best.reason.push_str(":primary_corner_batch_succeeded");
                            }
                            let prefix_len = best
                                .reason
                                .split("prefix_len=")
                                .nth(1)
                                .and_then(|tail| tail.split(':').next())
                                .and_then(|value| value.parse::<usize>().ok())
                                .unwrap_or(0);
                            (
                                true,
                                prefix_len,
                                best.raw_moves.len().saturating_sub(prefix_len),
                                best.raw_moves.len(),
                                best.optimized_moves.len(),
                                best.reason,
                            )
                        } else {
                            let suffix_reason = join_counts(&reasons);
                            let reason = if candidate_prefix_lens.is_empty() {
                                suffix_reason
                            } else {
                                format!(
                                    "phase_prefix_lens={candidate_prefix_lens}; {suffix_reason}"
                                )
                            };
                            (false, 0, 0, 0, 0, reason)
                        };

                        let delta = if total_found && direct.found {
                            total_opt_len as isize - direct_opt_len as isize
                        } else {
                            0
                        };
                        let record = PhaseLabRecord {
                            layout,
                            difficulty,
                            scramble_len,
                            iteration: iteration + 1,
                            seed: run_seed,
                            ariadne_solution_len,
                            phase_kind: spec.kind.label().to_string(),
                            phase_spec: phase_spec_label,
                            direct_found: direct.found,
                            direct_opt_len,
                            direct_elapsed_ms: direct.elapsed_ms,
                            phase_found: !prefixes.is_empty(),
                            suffix_found: total_found,
                            total_found,
                            prefix_len,
                            suffix_len,
                            total_raw_len,
                            total_opt_len,
                            delta_opt_vs_direct: delta,
                            phase_elapsed_ms,
                            suffix_elapsed_ms,
                            total_elapsed_ms: phase_elapsed_ms + suffix_elapsed_ms,
                            nodes: phase_nodes + suffix_nodes,
                            prefixes_available,
                            prefixes_tested,
                            candidate_min_prefix_len,
                            candidate_prefix_lens,
                            candidate_signatures,
                            reason,
                        };

                        if options.e_classic_cascade {
                            let stage_label = if corner_stage { "corner" } else { "near-pair" };
                            cascade_phase_found |= record.phase_found;
                            cascade_phase_elapsed_ms += record.phase_elapsed_ms;
                            cascade_suffix_elapsed_ms += record.suffix_elapsed_ms;
                            cascade_nodes += record.nodes;
                            if record.total_found
                                && (!cascade_found || record.total_opt_len < cascade_opt_len)
                            {
                                cascade_found = true;
                                cascade_selected_stage = stage_label;
                                cascade_raw_len = record.total_raw_len;
                                cascade_opt_len = record.total_opt_len;
                                cascade_prefix_len = record.prefix_len;
                                cascade_suffix_len = record.suffix_len;
                                cascade_prefixes_available = record.prefixes_available;
                                cascade_prefixes_tested = record.prefixes_tested;
                                cascade_candidate_min_prefix_len = record.candidate_min_prefix_len;
                                cascade_candidate_prefix_lens =
                                    record.candidate_prefix_lens.clone();
                                cascade_candidate_signatures = record.candidate_signatures.clone();
                            }
                            cascade_reasons.push(format!(
                                "{stage_label}:{}:{}",
                                if record.total_found {
                                    "found"
                                } else {
                                    "not_found"
                                },
                                record.reason
                            ));
                        }

                        println!(
                            "{layout}-{difficulty} scramble={scramble_len} iter={} spec={} direct={} total_found={} opt={} delta={} prefix={} suffix={} candidates=[{}] tested={}/{} time={}ms reason={}",
                            iteration + 1,
                            record.phase_spec,
                            record.direct_opt_len,
                            record.total_found,
                            if record.total_found { record.total_opt_len.to_string() } else { "-".to_string() },
                            record.delta_opt_vs_direct,
                            record.prefix_len,
                            record.suffix_len,
                            record.candidate_prefix_lens,
                            record.prefixes_tested,
                            record.prefixes_available,
                            record.total_elapsed_ms,
                            record.reason
                        );

                        records.push(record);
                    }
                    if options.e_classic_cascade {
                        if axis_ring_rescue_available
                            && options.axis_ring_rescue_position.runs_after_cascade()
                        {
                            if should_run_axis_ring_rescue(
                                &options,
                                &puzzle,
                                cascade_found,
                                cascade_opt_len,
                            ) {
                                let cascade_found_before_axis = cascade_found;
                                let cascade_opt_before_axis = cascade_opt_len;
                                let axis_record = run_axis_ring_rescue_record(
                                    &puzzle,
                                    &options,
                                    &mut axis_ring_rescue_artifacts,
                                    &artifacts,
                                    layout,
                                    difficulty,
                                    scramble_len,
                                    iteration,
                                    run_seed,
                                    ariadne_solution_len,
                                    &direct,
                                    cascade_found,
                                    cascade_opt_len,
                                    &scrambled_colors,
                                )?;
                                cascade_phase_found |= axis_record.phase_found;
                                cascade_suffix_elapsed_ms += axis_record.total_elapsed_ms;
                                cascade_nodes += axis_record.nodes;
                                if axis_record.total_found
                                    && (!cascade_found
                                        || axis_record.total_opt_len < cascade_opt_len)
                                {
                                    cascade_found = true;
                                    cascade_selected_stage = "axis-ring";
                                    cascade_raw_len = axis_record.total_raw_len;
                                    cascade_opt_len = axis_record.total_opt_len;
                                    cascade_prefix_len = axis_record.prefix_len;
                                    cascade_suffix_len = axis_record.suffix_len;
                                    cascade_prefixes_available = axis_record.prefixes_available;
                                    cascade_prefixes_tested = axis_record.prefixes_tested;
                                    cascade_candidate_min_prefix_len =
                                        axis_record.candidate_min_prefix_len;
                                    cascade_candidate_prefix_lens =
                                        axis_record.candidate_prefix_lens.clone();
                                    cascade_candidate_signatures =
                                        axis_record.candidate_signatures.clone();
                                }
                                cascade_reasons.push(format!(
                                    "axis-ring:{}:{}",
                                    if axis_record.total_found {
                                        "found"
                                    } else {
                                        "not_found"
                                    },
                                    axis_record.reason
                                ));
                                println!(
                                    "{layout}-{difficulty} scramble={scramble_len} iter={} axis_ring_rescue found={} opt={} cascade_before={} time={}ms nodes={} reason={}",
                                    iteration + 1,
                                    axis_record.total_found,
                                    if axis_record.total_found {
                                        axis_record.total_opt_len.to_string()
                                    } else {
                                        "-".to_string()
                                    },
                                    if cascade_found_before_axis {
                                        cascade_opt_before_axis.to_string()
                                    } else {
                                        "-".to_string()
                                    },
                                    axis_record.total_elapsed_ms,
                                    axis_record.nodes,
                                    axis_record.reason
                                );
                                records.push(axis_record);
                            } else {
                                cascade_reasons.push(format!(
                                    "axis-ring:skipped:threshold={}:current={}",
                                    options.axis_ring_rescue_threshold,
                                    if cascade_found {
                                        cascade_opt_len.to_string()
                                    } else {
                                        "-".to_string()
                                    }
                                ));
                            }
                        }
                        let delta_opt_vs_direct = if cascade_found && direct.found {
                            cascade_opt_len as isize - direct_opt_len as isize
                        } else {
                            0
                        };
                        let cascade_record = PhaseLabRecord {
                            layout,
                            difficulty,
                            scramble_len,
                            iteration: iteration + 1,
                            seed: run_seed,
                            ariadne_solution_len,
                            phase_kind: "e-classic-cascade".to_string(),
                            phase_spec: "direct->global-near-pair->quality-or-rescue-auto-corner"
                                .to_string(),
                            direct_found: direct.found,
                            direct_opt_len,
                            direct_elapsed_ms: direct.elapsed_ms,
                            phase_found: cascade_phase_found,
                            suffix_found: cascade_found && cascade_selected_stage != "direct",
                            total_found: cascade_found,
                            prefix_len: cascade_prefix_len,
                            suffix_len: cascade_suffix_len,
                            total_raw_len: cascade_raw_len,
                            total_opt_len: cascade_opt_len,
                            delta_opt_vs_direct,
                            phase_elapsed_ms: cascade_phase_elapsed_ms,
                            suffix_elapsed_ms: cascade_suffix_elapsed_ms,
                            total_elapsed_ms: direct.elapsed_ms
                                + cascade_phase_elapsed_ms
                                + cascade_suffix_elapsed_ms,
                            nodes: cascade_nodes,
                            prefixes_available: cascade_prefixes_available,
                            prefixes_tested: cascade_prefixes_tested,
                            candidate_min_prefix_len: cascade_candidate_min_prefix_len,
                            candidate_prefix_lens: cascade_candidate_prefix_lens,
                            candidate_signatures: cascade_candidate_signatures,
                            reason: format!(
                                "selected={cascade_selected_stage}; {}",
                                cascade_reasons.join("; ")
                            ),
                        };
                        println!(
                            "{layout}-{difficulty} scramble={scramble_len} iter={} cascade_found={} selected={} opt={} time={}ms nodes={}",
                            iteration + 1,
                            cascade_record.total_found,
                            cascade_selected_stage,
                            if cascade_record.total_found {
                                cascade_record.total_opt_len.to_string()
                            } else {
                                "-".to_string()
                            },
                            cascade_record.total_elapsed_ms,
                            cascade_record.nodes
                        );
                        records.push(cascade_record);
                    }
                }
            }
        }
    }

    write_phase_lab_csv(&csv_path, &records)?;
    write_phase_lab_markdown(&md_path, &options, &records)?;

    println!("CSV: {}", csv_path.display());
    println!("MD:  {}", md_path.display());
    Ok(())
}

fn artifact_summary(artifacts: &SolverArtifacts) -> String {
    let mut parts = artifacts
        .variants
        .iter()
        .map(|variant| {
            let rescue_ops = variant
                .rescue_operations
                .as_ref()
                .map_or(0, |operations| operations.len());
            let pattern_summary = variant.pattern_db.as_ref().map_or_else(
                || "off".to_string(),
                |pattern_db| {
                    format!(
                        "{}:depth{},states={},canonical={},weight={}",
                        pattern_db.projection.label(),
                        pattern_db.depth,
                        pattern_db.direct_len(),
                        pattern_db.canonical_len(),
                        pattern_db.weight
                    )
                },
            );
            let operation_sets = variant
                .operation_sets
                .iter()
                .map(|set| {
                    format!(
                        "{}:{}@{}ms",
                        set.profile.label(),
                        set.operations.len(),
                        set.time_limit_ms
                    )
                })
                .collect::<Vec<_>>()
                .join("+");
            format!(
                "{}:states={},ops=[{}],rescue_ops={},pattern={},nodes={},time={}ms",
                variant.target_mode.label(),
                variant.table.len(),
                operation_sets,
                rescue_ops,
                pattern_summary,
                variant.build_nodes,
                variant.build_ms
            )
        })
        .collect::<Vec<_>>();

    if let Some(pair_region) = &artifacts.pair_region {
        parts.push(format!(
            "pair-region:states={},ops={},preserve_ops={},nodes={},time={}ms",
            pair_region.table.len(),
            pair_region.operations.len(),
            pair_region.preserving_operations.len(),
            pair_region.build_nodes,
            pair_region.build_ms
        ));
    }

    parts.join("; ")
}

fn operation_sets_summary(operation_sets: &[OperationSetArtifacts]) -> String {
    operation_sets
        .iter()
        .map(|set| {
            format!(
                "{}:{}@{}ms",
                set.profile.label(),
                set.operations.len(),
                set.time_limit_ms
            )
        })
        .collect::<Vec<_>>()
        .join("+")
}

fn prepare_solver(puzzle: &Puzzle, config: &SolverConfig) -> io::Result<SolverArtifacts> {
    let started = Instant::now();
    let operation_sets = build_operation_sets(puzzle, config);
    let rescue_operations = build_rescue_operations(puzzle, config);
    let mut variants = Vec::new();
    let mut total_nodes = 0u64;

    for target_mode in target_variants(config.target_mode) {
        let mut variant_config = config.clone();
        variant_config.target_mode = target_mode;
        let variant_started = Instant::now();

        let mut pattern_nodes = 0u64;
        let mut pattern_limits = Limits::new(config.max_nodes, config.time_limit_ms);
        let pattern_db = build_pattern_db(
            puzzle,
            &variant_config,
            &mut pattern_nodes,
            &mut pattern_limits,
        );

        let mut limits = Limits::new(config.max_nodes, config.time_limit_ms);
        let mut nodes = 0u64;
        let table = build_reverse_table(puzzle, &variant_config, &mut nodes, &mut limits)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::TimedOut,
                    format!(
                        "failed to build reverse table for {}-{} target={}: {}",
                        puzzle.layout,
                        puzzle.difficulty,
                        target_mode.label(),
                        limits
                            .stop_reason
                            .clone()
                            .unwrap_or_else(|| "not_found".to_string())
                    ),
                )
            })?;
        let variant_nodes = nodes + pattern_nodes;
        total_nodes += variant_nodes;
        variants.push(SolverVariantArtifacts {
            target_mode,
            table,
            operation_sets: operation_sets.clone(),
            rescue_operations: rescue_operations.clone(),
            pattern_db,
            build_nodes: variant_nodes,
            build_ms: variant_started.elapsed().as_millis(),
        });
    }

    let pair_region = build_pair_region_artifacts(puzzle, config);
    if let Some(pair_region) = &pair_region {
        total_nodes += pair_region.build_nodes;
    }

    Ok(SolverArtifacts {
        variants,
        pair_region,
        build_nodes: total_nodes,
        build_ms: started.elapsed().as_millis(),
    })
}

fn beam_rank_execution_modes(mode: BeamRankMode) -> Vec<BeamRankMode> {
    match mode {
        BeamRankMode::RingPortfolio => vec![BeamRankMode::Score, BeamRankMode::RingRotation],
        other => vec![other],
    }
}

fn portfolio_rank_reason_label(
    requested: BeamRankMode,
    actual: BeamRankMode,
) -> Option<&'static str> {
    if requested == BeamRankMode::RingPortfolio {
        Some(actual.label())
    } else {
        None
    }
}

fn found_reason_for_rank(
    use_pattern_db: bool,
    requested: BeamRankMode,
    actual: BeamRankMode,
) -> String {
    match (use_pattern_db, requested == BeamRankMode::RingPortfolio) {
        (true, true) => format!("found_pattern_{}", actual.label()),
        (false, true) => format!("found_{}", actual.label()),
        (true, false) => "found_pattern".to_string(),
        (false, false) => "found".to_string(),
    }
}

fn hard_rescue_reason_for_rank(
    is_quality_rescue: bool,
    requested: BeamRankMode,
    actual: BeamRankMode,
) -> String {
    let base = if is_quality_rescue {
        "found_quality_hard_rescue"
    } else {
        "found_hard_rescue"
    };
    if requested == BeamRankMode::RingPortfolio {
        format!("{}_{}", base, actual.label())
    } else {
        base.to_string()
    }
}

fn failure_reason_for_rank(
    prefix: &str,
    reason: &str,
    requested: BeamRankMode,
    actual: BeamRankMode,
) -> String {
    if let Some(rank_label) = portfolio_rank_reason_label(requested, actual) {
        format!("{prefix}:{rank_label}:{reason}")
    } else {
        format!("{prefix}:{reason}")
    }
}

fn solve_puzzle(
    puzzle: &Puzzle,
    colors: &[Color],
    config: &SolverConfig,
    artifacts: &SolverArtifacts,
) -> SolveResult {
    let started = Instant::now();
    let mut best: Option<SolveResult> = None;
    let mut primary_best_len: Option<usize> = None;
    let mut total_nodes = 0u64;
    let mut reasons: BTreeMap<String, usize> = BTreeMap::new();
    let acceptance_target = acceptance_target(config.target_mode);

    for variant in &artifacts.variants {
        let mut variant_best: Option<SolveResult> = None;
        for (operation_index, operation_set) in variant.operation_sets.iter().enumerate() {
            if operation_index > 0
                && !should_try_operation_portfolio_profile(puzzle, config, variant_best.as_ref())
            {
                continue;
            }

            for use_pattern_db in [false, true] {
                let pattern_db = if use_pattern_db {
                    if should_try_f_pattern_db_portfolio(puzzle, config) {
                        continue;
                    }
                    if config.beam_rank.needs_pattern_db() {
                        continue;
                    }
                    if operation_index > 0 {
                        continue;
                    }
                    if !should_try_pattern_db_profile(config, best.as_ref()) {
                        continue;
                    }
                    match variant.pattern_db.as_ref() {
                        Some(pattern_db) => Some(pattern_db),
                        None => continue,
                    }
                } else {
                    match config.beam_rank.needs_pattern_db() {
                        true => variant.pattern_db.as_ref(),
                        false => None,
                    }
                };

                for rank_mode in beam_rank_execution_modes(config.beam_rank) {
                    let mut variant_config = config.clone();
                    variant_config.target_mode = variant.target_mode;
                    variant_config.operation_profile = operation_set.profile;
                    variant_config.time_limit_ms = operation_set.time_limit_ms;
                    variant_config.beam_rank = rank_mode;
                    let mut limits = Limits::new(config.max_nodes, operation_set.time_limit_ms);
                    let mut nodes = 0u64;
                    let mut first_table_hit = None;

                    match solve_input_traced(
                        puzzle,
                        colors,
                        &variant_config,
                        &variant.table,
                        &operation_set.operations,
                        pattern_db,
                        &mut nodes,
                        &mut limits,
                        &mut first_table_hit,
                    ) {
                        Some((method, raw_moves)) => {
                            total_nodes += nodes;
                            let optimized_moves = if config.optimize {
                                let (local_window, local_depth) = local_optimization_for_profile(
                                    puzzle,
                                    config,
                                    operation_set.profile,
                                );
                                optimize_solution_with_table(
                                    puzzle,
                                    colors,
                                    &raw_moves,
                                    &variant.table,
                                    local_window,
                                    local_depth,
                                )
                            } else {
                                raw_moves.clone()
                            };

                            if !solution_matches_target(
                                puzzle,
                                colors,
                                &optimized_moves,
                                acceptance_target,
                            ) {
                                *reasons.entry("invalid_target".to_string()).or_insert(0) += 1;
                                continue;
                            }

                            let candidate = SolveResult {
                                found: true,
                                method,
                                target_used: Some(variant.target_mode),
                                operation_profile_used: Some(operation_set.profile),
                                reason: found_reason_for_rank(
                                    use_pattern_db,
                                    config.beam_rank,
                                    rank_mode,
                                ),
                                raw_moves,
                                optimized_moves,
                                nodes,
                                elapsed_ms: 0,
                                first_table_hit,
                            };
                            if config.portfolio_first_result {
                                let mut candidate = candidate;
                                candidate.reason =
                                    format!("{}:portfolio_first_result", candidate.reason);
                                candidate.nodes = total_nodes;
                                candidate.elapsed_ms = started.elapsed().as_millis();
                                return candidate;
                            }
                            if operation_index == 0 {
                                let candidate_len = candidate.optimized_moves.len();
                                primary_best_len =
                                    Some(primary_best_len.map_or(candidate_len, |current| {
                                        current.min(candidate_len)
                                    }));
                            }
                            keep_best_result(&mut variant_best, candidate.clone());
                            keep_best_result(&mut best, candidate);
                        }
                        None => {
                            total_nodes += nodes;
                            let reason = limits
                                .stop_reason
                                .clone()
                                .unwrap_or_else(|| "not_found".to_string());
                            let profile_label = if use_pattern_db {
                                format!("{}+pattern", operation_set.profile.label())
                            } else {
                                operation_set.profile.label().to_string()
                            };
                            *reasons
                                .entry(failure_reason_for_rank(
                                    &profile_label,
                                    &reason,
                                    config.beam_rank,
                                    rank_mode,
                                ))
                                .or_insert(0) += 1;
                        }
                    }
                }
            }
        }
    }

    if should_try_f_pattern_db_portfolio(puzzle, config) {
        for variant in &artifacts.variants {
            let Some(pattern_db) = variant.pattern_db.as_ref() else {
                *reasons
                    .entry(format!(
                        "f-pattern-db:{}:missing",
                        variant.target_mode.label()
                    ))
                    .or_insert(0) += 1;
                continue;
            };

            let mut rank_variant_best: Option<SolveResult> = None;
            for (operation_index, operation_set) in variant.operation_sets.iter().enumerate() {
                if operation_index > 0
                    && !should_try_operation_portfolio_profile(
                        puzzle,
                        config,
                        rank_variant_best.as_ref(),
                    )
                {
                    continue;
                }

                let mut variant_config = config.clone();
                variant_config.target_mode = variant.target_mode;
                variant_config.operation_profile = operation_set.profile;
                variant_config.time_limit_ms = operation_set.time_limit_ms;
                variant_config.beam_rank = BeamRankMode::PatternDistance;

                let mut limits = Limits::new(config.max_nodes, operation_set.time_limit_ms);
                let mut nodes = 0u64;
                match solve_input(
                    puzzle,
                    colors,
                    &variant_config,
                    &variant.table,
                    &operation_set.operations,
                    Some(pattern_db),
                    &mut nodes,
                    &mut limits,
                ) {
                    Some((method, raw_moves)) => {
                        total_nodes += nodes;
                        let optimized_moves = if config.optimize {
                            let (local_window, local_depth) = local_optimization_for_profile(
                                puzzle,
                                config,
                                operation_set.profile,
                            );
                            optimize_solution_with_table(
                                puzzle,
                                colors,
                                &raw_moves,
                                &variant.table,
                                local_window,
                                local_depth,
                            )
                        } else {
                            raw_moves.clone()
                        };

                        if !solution_matches_target(
                            puzzle,
                            colors,
                            &optimized_moves,
                            acceptance_target,
                        ) {
                            *reasons
                                .entry("f-pattern-db:invalid_target".to_string())
                                .or_insert(0) += 1;
                            continue;
                        }

                        let candidate = SolveResult {
                            found: true,
                            method,
                            target_used: Some(variant.target_mode),
                            operation_profile_used: Some(operation_set.profile),
                            reason: "found_f_pattern_db".to_string(),
                            raw_moves,
                            optimized_moves,
                            nodes,
                            elapsed_ms: 0,
                            first_table_hit: None,
                        };
                        keep_best_result(&mut rank_variant_best, candidate.clone());
                        keep_best_result(&mut best, candidate);
                    }
                    None => {
                        total_nodes += nodes;
                        let reason = limits
                            .stop_reason
                            .clone()
                            .unwrap_or_else(|| "not_found".to_string());
                        *reasons
                            .entry(format!(
                                "f-pattern-db:{}:{}",
                                operation_set.profile.label(),
                                reason
                            ))
                            .or_insert(0) += 1;
                    }
                }
            }
        }
    }

    if should_try_pair_region_rescue(puzzle, config, best.as_ref(), artifacts) {
        let (candidate, pair_nodes, reason) =
            try_pair_region_rescue(puzzle, colors, config, artifacts, acceptance_target);
        total_nodes += pair_nodes;
        if let Some(candidate) = candidate {
            keep_best_result(&mut best, candidate);
        } else {
            *reasons.entry(reason).or_insert(0) += 1;
        }
    }

    if let Some(hard_rescue_time_limit_ms) =
        hard_rescue_time_limit_for(puzzle, config, best.as_ref())
    {
        let is_quality_rescue = best.is_some();
        for variant in &artifacts.variants {
            if variant.target_mode != TargetMode::AndroidMultiGoal {
                continue;
            }

            let hard_operations_storage;
            let hard_operations = if let Some(operation_set) = variant
                .operation_sets
                .iter()
                .find(|set| set.profile == OperationProfile::ExpandedWide)
            {
                &operation_set.operations
            } else {
                hard_operations_storage = build_operations(puzzle, OperationProfile::ExpandedWide);
                &hard_operations_storage
            };

            for rank_mode in beam_rank_execution_modes(config.beam_rank) {
                let mut hard_config = config.clone();
                hard_config.target_mode = variant.target_mode;
                hard_config.operation_profile = OperationProfile::ExpandedWide;
                hard_config.tiers = vec![hard_rescue_tier_for(puzzle, config, best.as_ref())];
                hard_config.time_limit_ms = hard_rescue_time_limit_ms;
                hard_config.beam_rank = rank_mode;
                let mut hard_limits = Limits::new(config.max_nodes, hard_rescue_time_limit_ms);
                let mut hard_nodes = 0u64;
                let hard_result = solve_input(
                    puzzle,
                    colors,
                    &hard_config,
                    &variant.table,
                    hard_operations,
                    variant.pattern_db.as_ref(),
                    &mut hard_nodes,
                    &mut hard_limits,
                );
                total_nodes += hard_nodes;

                match hard_result {
                    Some((method, raw_moves)) => {
                        let optimized_moves = if config.optimize {
                            optimize_solution_with_table(
                                puzzle,
                                colors,
                                &raw_moves,
                                &variant.table,
                                config.local_window,
                                config.local_depth,
                            )
                        } else {
                            raw_moves.clone()
                        };

                        if solution_matches_target(
                            puzzle,
                            colors,
                            &optimized_moves,
                            acceptance_target,
                        ) {
                            let candidate = SolveResult {
                                found: true,
                                method,
                                target_used: Some(variant.target_mode),
                                operation_profile_used: Some(OperationProfile::ExpandedWide),
                                reason: hard_rescue_reason_for_rank(
                                    is_quality_rescue,
                                    config.beam_rank,
                                    rank_mode,
                                ),
                                raw_moves,
                                optimized_moves,
                                nodes: hard_nodes,
                                elapsed_ms: 0,
                                first_table_hit: None,
                            };
                            keep_best_result(&mut best, candidate);
                        } else {
                            *reasons
                                .entry("hard-rescue:invalid_target".to_string())
                                .or_insert(0) += 1;
                        }
                    }
                    None => {
                        let reason = hard_limits
                            .stop_reason
                            .clone()
                            .unwrap_or_else(|| "not_found".to_string());
                        let label = if is_quality_rescue {
                            "quality-hard-rescue"
                        } else {
                            "hard-rescue"
                        };
                        *reasons
                            .entry(failure_reason_for_rank(
                                label,
                                &reason,
                                config.beam_rank,
                                rank_mode,
                            ))
                            .or_insert(0) += 1;
                    }
                }
            }
        }
    }

    if should_try_landmark_rescue(puzzle, config, best.as_ref()) {
        for variant in &artifacts.variants {
            let Some(pattern_db) = variant.pattern_db.as_ref() else {
                continue;
            };
            let Some(operation_set) = variant.operation_sets.first() else {
                continue;
            };

            let mut landmark_config = config.clone();
            landmark_config.target_mode = variant.target_mode;
            landmark_config.operation_profile = operation_set.profile;
            landmark_config.time_limit_ms = config.landmark_time_limit_ms;
            let mut landmark_limits = Limits::new(config.max_nodes, config.landmark_time_limit_ms);
            let mut landmark_nodes = 0u64;
            let landmark_result = try_landmark_rescue(
                puzzle,
                colors,
                &landmark_config,
                &variant.table,
                &operation_set.operations,
                pattern_db,
                &mut landmark_nodes,
                &mut landmark_limits,
            );
            total_nodes += landmark_nodes;

            match landmark_result {
                Some((method, raw_moves)) => {
                    let optimized_moves = if config.optimize {
                        optimize_solution_with_table(
                            puzzle,
                            colors,
                            &raw_moves,
                            &variant.table,
                            config.local_window,
                            config.local_depth,
                        )
                    } else {
                        raw_moves.clone()
                    };

                    if solution_matches_target(puzzle, colors, &optimized_moves, acceptance_target)
                    {
                        let candidate = SolveResult {
                            found: true,
                            method,
                            target_used: Some(variant.target_mode),
                            operation_profile_used: Some(operation_set.profile),
                            reason: "found_landmark".to_string(),
                            raw_moves,
                            optimized_moves,
                            nodes: landmark_nodes,
                            elapsed_ms: 0,
                            first_table_hit: None,
                        };
                        keep_best_result(&mut best, candidate);
                    } else {
                        *reasons
                            .entry("landmark:invalid_target".to_string())
                            .or_insert(0) += 1;
                    }
                }
                None => {
                    let reason = landmark_limits
                        .stop_reason
                        .clone()
                        .unwrap_or_else(|| "not_found".to_string());
                    *reasons.entry(format!("landmark:{reason}")).or_insert(0) += 1;
                }
            }
        }
    }

    if should_try_rescue(puzzle, config, primary_best_len, best.as_ref()) {
        for variant in &artifacts.variants {
            let Some(rescue_operations) = &variant.rescue_operations else {
                continue;
            };

            let mut rescue_config = config.clone();
            rescue_config.target_mode = variant.target_mode;
            rescue_config.time_limit_ms = config.rescue_time_limit_ms;
            let mut rescue_limits = Limits::new(config.max_nodes, config.rescue_time_limit_ms);
            let mut rescue_nodes = 0u64;
            if let Some((rescue_method, rescue_raw_moves)) = solve_input(
                puzzle,
                colors,
                &rescue_config,
                &variant.table,
                rescue_operations,
                None,
                &mut rescue_nodes,
                &mut rescue_limits,
            ) {
                total_nodes += rescue_nodes;
                let rescue_optimized_moves = if config.optimize {
                    optimize_solution_with_table(
                        puzzle,
                        colors,
                        &rescue_raw_moves,
                        &variant.table,
                        F_RESCUE_LOCAL_WINDOW,
                        F_RESCUE_LOCAL_DEPTH,
                    )
                } else {
                    rescue_raw_moves.clone()
                };

                if solution_matches_target(
                    puzzle,
                    colors,
                    &rescue_optimized_moves,
                    acceptance_target,
                ) {
                    let rescue_candidate = SolveResult {
                        found: true,
                        method: rescue_method,
                        target_used: Some(variant.target_mode),
                        operation_profile_used: Some(OperationProfile::Pairs),
                        reason: "found_rescue".to_string(),
                        raw_moves: rescue_raw_moves,
                        optimized_moves: rescue_optimized_moves,
                        nodes: rescue_nodes,
                        elapsed_ms: 0,
                        first_table_hit: None,
                    };
                    keep_best_result(&mut best, rescue_candidate);
                }
            } else {
                total_nodes += rescue_nodes;
            }
        }
    }

    if let Some(mut best) = best {
        best.nodes = total_nodes;
        best.elapsed_ms = started.elapsed().as_millis();
        return best;
    }

    SolveResult {
        found: false,
        method: SolveMethod::Macro,
        target_used: None,
        operation_profile_used: None,
        reason: if reasons.is_empty() {
            "not_found".to_string()
        } else {
            join_counts(&reasons)
        },
        raw_moves: Vec::new(),
        optimized_moves: Vec::new(),
        nodes: total_nodes,
        elapsed_ms: started.elapsed().as_millis(),
        first_table_hit: None,
    }
}

fn solve_input(
    puzzle: &Puzzle,
    colors: &[Color],
    config: &SolverConfig,
    table: &HashMap<Key, PathBits>,
    operations: &[Operation],
    pattern_db: Option<&PatternDb>,
    nodes: &mut u64,
    limits: &mut Limits,
) -> Option<(SolveMethod, Vec<MoveIndex>)> {
    let mut first_table_hit = None;
    solve_input_traced(
        puzzle,
        colors,
        config,
        table,
        operations,
        pattern_db,
        nodes,
        limits,
        &mut first_table_hit,
    )
}

fn solve_input_traced(
    puzzle: &Puzzle,
    colors: &[Color],
    config: &SolverConfig,
    table: &HashMap<Key, PathBits>,
    operations: &[Operation],
    pattern_db: Option<&PatternDb>,
    nodes: &mut u64,
    limits: &mut Limits,
    first_table_hit: &mut Option<TableHitTelemetry>,
) -> Option<(SolveMethod, Vec<MoveIndex>)> {
    if is_target_solved(colors, puzzle, config.target_mode) {
        return Some((SolveMethod::None, Vec::new()));
    }

    if let Some(moves) = meet_in_middle(puzzle, colors, table, config, nodes, limits) {
        return Some((SolveMethod::Mitm, moves));
    }

    for &tier in &config.tiers {
        if limits.exceeded(*nodes) {
            return None;
        }
        if let Some(moves) = macro_beam(
            puzzle,
            colors,
            table,
            operations,
            pattern_db,
            tier,
            config,
            nodes,
            limits,
            first_table_hit,
        ) {
            return Some((SolveMethod::Macro, moves));
        }
    }

    None
}

fn build_pair_region_artifacts(
    puzzle: &Puzzle,
    config: &SolverConfig,
) -> Option<PairRegionArtifacts> {
    if !should_prepare_pair_region_rescue(puzzle, config) {
        return None;
    }

    let started = Instant::now();
    let mut pair_config = config.clone();
    pair_config.target_mode = TargetMode::PairRegion;
    pair_config.table_depth = Some(config.pair_region_table_depth);
    pair_config.forward_depth = Some(config.pair_region_forward_depth);
    pair_config.operation_profile = OperationProfile::ExpandedParallel;
    pair_config.tiers = vec![config.pair_region_tier];
    pair_config.time_limit_ms = config.pair_region_time_limit_ms;

    let mut nodes = 0u64;
    let mut limits = Limits::new(config.max_nodes, config.pair_region_time_limit_ms);
    let table = build_reverse_table(puzzle, &pair_config, &mut nodes, &mut limits)?;
    let operations = build_operations(puzzle, OperationProfile::ExpandedParallel);
    let preserving_operations =
        build_pair_region_preserving_operations(puzzle, OperationProfile::ExpandedWide);

    Some(PairRegionArtifacts {
        table,
        operations,
        preserving_operations,
        build_nodes: nodes,
        build_ms: started.elapsed().as_millis(),
    })
}

fn build_pair_region_preserving_operations(
    puzzle: &Puzzle,
    profile: OperationProfile,
) -> Vec<Operation> {
    let seeds = generate_pair_region_solved_states(puzzle);
    build_operations(puzzle, profile)
        .into_iter()
        .filter(|operation| {
            seeds.iter().all(|seed| {
                let mut colors = seed.clone();
                for &move_index in &operation.moves {
                    puzzle.apply_move(&mut colors, move_index);
                }
                is_pair_region_solved(&colors, puzzle)
            })
        })
        .collect()
}

fn try_pair_region_rescue(
    puzzle: &Puzzle,
    colors: &[Color],
    config: &SolverConfig,
    artifacts: &SolverArtifacts,
    acceptance_target: TargetMode,
) -> (Option<SolveResult>, u64, String) {
    let Some(pair_artifacts) = artifacts.pair_region.as_ref() else {
        return (None, 0, "pair-region:not_prepared".to_string());
    };

    let mut total_nodes = 0u64;
    let mut reasons: BTreeMap<String, usize> = BTreeMap::new();
    let pair_colors = pair_region_colors(puzzle, colors);

    let mut pair_config = config.clone();
    pair_config.target_mode = TargetMode::PairRegion;
    pair_config.table_depth = Some(config.pair_region_table_depth);
    pair_config.forward_depth = Some(config.pair_region_forward_depth);
    pair_config.operation_profile = OperationProfile::ExpandedParallel;
    pair_config.tiers = vec![config.pair_region_tier];
    pair_config.time_limit_ms = config.pair_region_time_limit_ms;
    let mut pair_limits = Limits::new(config.max_nodes, config.pair_region_time_limit_ms);
    let mut pair_nodes = 0u64;
    let pair_prefixes = collect_pair_region_prefixes(
        puzzle,
        &pair_colors,
        &pair_config,
        &pair_artifacts.table,
        &pair_artifacts.operations,
        None,
        config.pair_region_prefixes,
        &mut pair_nodes,
        &mut pair_limits,
    );
    if pair_prefixes.is_empty() {
        total_nodes += pair_nodes;
        let reason = pair_limits
            .stop_reason
            .clone()
            .unwrap_or_else(|| "not_found".to_string());
        return (None, total_nodes, format!("pair-region-phase1:{reason}"));
    }
    total_nodes += pair_nodes;
    let pair_prefix_lens = pair_prefixes
        .iter()
        .map(|prefix| prefix.len().to_string())
        .collect::<Vec<_>>()
        .join(",");

    let has_android_multi = artifacts
        .variants
        .iter()
        .any(|variant| variant.target_mode == TargetMode::AndroidMultiGoal);
    let mut best: Option<SolveResult> = None;

    for (prefix_index, pair_prefix) in pair_prefixes.iter().enumerate() {
        let mut intermediate_colors = colors.to_vec();
        puzzle.apply_moves(&mut intermediate_colors, pair_prefix);

        for variant in &artifacts.variants {
            if has_android_multi && variant.target_mode != TargetMode::AndroidMultiGoal {
                continue;
            }

            for (operation_index, operation_set) in variant.operation_sets.iter().enumerate() {
                let use_preserving = config.pair_region_preserve_suffix
                    && !pair_artifacts.preserving_operations.is_empty();
                if use_preserving && operation_index > 0 {
                    continue;
                }

                let suffix_operations = if use_preserving {
                    &pair_artifacts.preserving_operations
                } else {
                    &operation_set.operations
                };
                let suffix_profile = if use_preserving {
                    OperationProfile::ExpandedWide
                } else {
                    operation_set.profile
                };

                let mut suffix_config = config.clone();
                suffix_config.target_mode = variant.target_mode;
                suffix_config.operation_profile = suffix_profile;
                suffix_config.time_limit_ms = config.pair_region_suffix_time_limit_ms;
                let mut suffix_limits =
                    Limits::new(config.max_nodes, config.pair_region_suffix_time_limit_ms);
                let mut suffix_nodes = 0u64;

                match solve_input(
                    puzzle,
                    &intermediate_colors,
                    &suffix_config,
                    &variant.table,
                    suffix_operations,
                    variant.pattern_db.as_ref(),
                    &mut suffix_nodes,
                    &mut suffix_limits,
                ) {
                    Some((method, suffix_moves)) => {
                        total_nodes += suffix_nodes;
                        let mut raw_moves = pair_prefix.clone();
                        raw_moves.extend(suffix_moves);
                        let optimized_moves = if config.optimize {
                            optimize_solution_with_table(
                                puzzle,
                                colors,
                                &raw_moves,
                                &variant.table,
                                config.local_window,
                                config.local_depth,
                            )
                        } else {
                            raw_moves.clone()
                        };

                        if solution_matches_target(
                            puzzle,
                            colors,
                            &optimized_moves,
                            acceptance_target,
                        ) {
                            let candidate = SolveResult {
                                found: true,
                                method,
                                target_used: Some(variant.target_mode),
                                operation_profile_used: Some(suffix_profile),
                                reason: format!(
                                    "found_pair_region:prefix_index={prefix_index}:prefix_len={}",
                                    pair_prefix.len()
                                ),
                                raw_moves,
                                optimized_moves,
                                nodes: total_nodes,
                                elapsed_ms: 0,
                                first_table_hit: None,
                            };
                            keep_best_result(&mut best, candidate);
                        } else {
                            *reasons
                                .entry("pair-region-suffix:invalid_target".to_string())
                                .or_insert(0) += 1;
                        }
                    }
                    None => {
                        total_nodes += suffix_nodes;
                        let reason = suffix_limits
                            .stop_reason
                            .clone()
                            .unwrap_or_else(|| "not_found".to_string());
                        *reasons
                            .entry(format!(
                                "pair-region-suffix:{}:{reason}",
                                suffix_profile.label()
                            ))
                            .or_insert(0) += 1;
                    }
                }
            }
        }
    }

    let reason = if reasons.is_empty() {
        format!("pair-region:prefix_lens={pair_prefix_lens}:not_found")
    } else {
        format!(
            "pair-region:prefix_lens={pair_prefix_lens}:{}",
            join_counts(&reasons)
        )
    };
    (best, total_nodes, reason)
}

fn should_prepare_pair_region_rescue(puzzle: &Puzzle, config: &SolverConfig) -> bool {
    config.pair_region_rescue_enabled
        && config.pair_region_table_depth > 0
        && config.pair_region_forward_depth > 0
        && config.pair_region_time_limit_ms > 0
        && config.pair_region_suffix_time_limit_ms > 0
        && config.pair_region_tier.max_depth > 0
        && config.pair_region_tier.width > 0
        && config.pair_region_tier.restarts > 0
        && puzzle.layout == LayoutId::E
        && puzzle.difficulty == Difficulty::Classic
}

fn should_try_pair_region_rescue(
    puzzle: &Puzzle,
    config: &SolverConfig,
    best: Option<&SolveResult>,
    artifacts: &SolverArtifacts,
) -> bool {
    best.is_none()
        && artifacts.pair_region.is_some()
        && should_prepare_pair_region_rescue(puzzle, config)
}

fn solve_suffix_from_prefix(
    puzzle: &Puzzle,
    original_colors: &[Color],
    intermediate_colors: &[Color],
    prefix: &[MoveIndex],
    prefix_index: usize,
    config: &SolverConfig,
    artifacts: &SolverArtifacts,
    suffix_time_limit_ms: u64,
    hard_rescue_enabled: bool,
) -> SolveResult {
    let started = Instant::now();
    let acceptance_target = acceptance_target(config.target_mode);
    let mut best: Option<SolveResult> = None;
    let mut total_nodes = 0u64;
    let mut reasons = BTreeMap::new();

    for variant in &artifacts.variants {
        let mut variant_best: Option<SolveResult> = None;
        for (operation_index, operation_set) in variant.operation_sets.iter().enumerate() {
            if operation_index > 0
                && !should_try_operation_portfolio_profile(puzzle, config, variant_best.as_ref())
            {
                continue;
            }

            let limit_ms = if suffix_time_limit_ms == 0 {
                operation_set.time_limit_ms
            } else {
                operation_set.time_limit_ms.min(suffix_time_limit_ms).max(1)
            };
            let mut suffix_config = config.clone();
            suffix_config.target_mode = variant.target_mode;
            suffix_config.operation_profile = operation_set.profile;
            suffix_config.time_limit_ms = limit_ms;
            let mut suffix_nodes = 0u64;
            let mut suffix_limits = Limits::new(config.max_nodes, limit_ms);

            match solve_input(
                puzzle,
                intermediate_colors,
                &suffix_config,
                &variant.table,
                &operation_set.operations,
                variant.pattern_db.as_ref(),
                &mut suffix_nodes,
                &mut suffix_limits,
            ) {
                Some((method, suffix_moves)) => {
                    total_nodes += suffix_nodes;
                    let mut raw_moves = prefix.to_vec();
                    raw_moves.extend(suffix_moves);
                    let optimized_moves = if config.optimize {
                        let (local_window, local_depth) =
                            local_optimization_for_profile(puzzle, config, operation_set.profile);
                        optimize_solution_with_table(
                            puzzle,
                            original_colors,
                            &raw_moves,
                            &variant.table,
                            local_window,
                            local_depth,
                        )
                    } else {
                        raw_moves.clone()
                    };

                    if solution_matches_target(
                        puzzle,
                        original_colors,
                        &optimized_moves,
                        acceptance_target,
                    ) {
                        let candidate = SolveResult {
                            found: true,
                            method,
                            target_used: Some(variant.target_mode),
                            operation_profile_used: Some(operation_set.profile),
                            reason: format!(
                                "found_phase:prefix_index={prefix_index}:prefix_len={}",
                                prefix.len()
                            ),
                            raw_moves,
                            optimized_moves,
                            nodes: total_nodes,
                            elapsed_ms: started.elapsed().as_millis(),
                            first_table_hit: None,
                        };
                        keep_best_result(&mut variant_best, candidate.clone());
                        keep_best_result(&mut best, candidate);
                    } else {
                        *reasons
                            .entry("suffix:invalid_target".to_string())
                            .or_insert(0) += 1;
                    }
                }
                None => {
                    total_nodes += suffix_nodes;
                    let reason = suffix_limits
                        .stop_reason
                        .clone()
                        .unwrap_or_else(|| "not_found".to_string());
                    *reasons
                        .entry(format!("suffix:{}:{reason}", operation_set.profile.label()))
                        .or_insert(0) += 1;
                }
            }
        }
    }

    if hard_rescue_enabled
        && puzzle.layout == LayoutId::E
        && puzzle.difficulty == Difficulty::Classic
        && config.hard_rescue_time_limit_ms > 0
        && config.hard_rescue_tier.max_depth > 0
        && config.hard_rescue_tier.width > 0
        && config.hard_rescue_tier.restarts > 0
    {
        for variant in &artifacts.variants {
            if variant.target_mode != TargetMode::AndroidMultiGoal {
                continue;
            }

            let hard_operations_storage;
            let hard_operations = if let Some(operation_set) = variant
                .operation_sets
                .iter()
                .find(|set| set.profile == OperationProfile::ExpandedWide)
            {
                &operation_set.operations
            } else {
                hard_operations_storage = build_operations(puzzle, OperationProfile::ExpandedWide);
                &hard_operations_storage
            };
            let mut hard_config = config.clone();
            hard_config.target_mode = variant.target_mode;
            hard_config.operation_profile = OperationProfile::ExpandedWide;
            hard_config.tiers = vec![config.hard_rescue_tier];
            hard_config.time_limit_ms = config.hard_rescue_time_limit_ms;
            let mut hard_limits = Limits::new(config.max_nodes, config.hard_rescue_time_limit_ms);
            let mut hard_nodes = 0u64;

            match solve_input(
                puzzle,
                intermediate_colors,
                &hard_config,
                &variant.table,
                hard_operations,
                variant.pattern_db.as_ref(),
                &mut hard_nodes,
                &mut hard_limits,
            ) {
                Some((method, suffix_moves)) => {
                    total_nodes += hard_nodes;
                    let mut raw_moves = prefix.to_vec();
                    raw_moves.extend(suffix_moves);
                    let optimized_moves = if config.optimize {
                        optimize_solution_with_table(
                            puzzle,
                            original_colors,
                            &raw_moves,
                            &variant.table,
                            config.local_window,
                            config.local_depth,
                        )
                    } else {
                        raw_moves.clone()
                    };

                    if solution_matches_target(
                        puzzle,
                        original_colors,
                        &optimized_moves,
                        acceptance_target,
                    ) {
                        let candidate = SolveResult {
                            found: true,
                            method,
                            target_used: Some(variant.target_mode),
                            operation_profile_used: Some(OperationProfile::ExpandedWide),
                            reason: format!(
                                "found_phase_hard_rescue:prefix_index={prefix_index}:prefix_len={}",
                                prefix.len()
                            ),
                            raw_moves,
                            optimized_moves,
                            nodes: hard_nodes,
                            elapsed_ms: started.elapsed().as_millis(),
                            first_table_hit: None,
                        };
                        keep_best_result(&mut best, candidate);
                    } else {
                        *reasons
                            .entry("suffix-hard-rescue:invalid_target".to_string())
                            .or_insert(0) += 1;
                    }
                }
                None => {
                    total_nodes += hard_nodes;
                    let reason = hard_limits
                        .stop_reason
                        .clone()
                        .unwrap_or_else(|| "not_found".to_string());
                    *reasons
                        .entry(format!("suffix-hard-rescue:{reason}"))
                        .or_insert(0) += 1;
                }
            }
        }
    }

    if let Some(mut best) = best {
        best.nodes = total_nodes;
        best.elapsed_ms = started.elapsed().as_millis();
        return best;
    }

    SolveResult {
        found: false,
        method: SolveMethod::Macro,
        target_used: None,
        operation_profile_used: None,
        reason: if reasons.is_empty() {
            "suffix:not_found".to_string()
        } else {
            join_counts(&reasons)
        },
        raw_moves: Vec::new(),
        optimized_moves: Vec::new(),
        nodes: total_nodes,
        elapsed_ms: started.elapsed().as_millis(),
        first_table_hit: None,
    }
}

fn try_landmark_rescue(
    puzzle: &Puzzle,
    colors: &[Color],
    config: &SolverConfig,
    table: &HashMap<Key, PathBits>,
    operations: &[Operation],
    pattern_db: &PatternDb,
    nodes: &mut u64,
    limits: &mut Limits,
) -> Option<(SolveMethod, Vec<MoveIndex>)> {
    let candidates = find_landmark_candidates(
        puzzle, colors, operations, pattern_db, config, nodes, limits,
    );
    let candidate_count = candidates.len();
    let best_distance = candidates
        .iter()
        .map(|candidate| candidate.pattern_distance)
        .min();
    let mut best_hit: Option<Vec<MoveIndex>> = None;

    for candidate in candidates {
        if limits.exceeded(*nodes) {
            break;
        }
        let elapsed = limits.started.elapsed();
        if elapsed >= limits.time_limit {
            limits.stop_reason = Some("timeout".to_string());
            break;
        }
        let remaining_ms = (limits.time_limit - elapsed)
            .as_millis()
            .min(u64::MAX as u128) as u64;
        let suffix_time_limit_ms = config
            .landmark_suffix_time_limit_ms
            .min(remaining_ms)
            .max(1);

        let mut suffix_config = config.clone();
        suffix_config.hit_patience = suffix_config.hit_patience.max(1);
        suffix_config.time_limit_ms = suffix_time_limit_ms;
        let mut suffix_nodes = 0u64;
        let mut suffix_limits = Limits::new(
            config.max_nodes.saturating_sub(*nodes),
            suffix_time_limit_ms,
        );
        if let Some((_, suffix)) = solve_input(
            puzzle,
            &candidate.colors,
            &suffix_config,
            table,
            operations,
            Some(pattern_db),
            &mut suffix_nodes,
            &mut suffix_limits,
        ) {
            *nodes += suffix_nodes;
            let mut path = candidate.path;
            path.extend(suffix);
            keep_shortest(&mut best_hit, path);
        } else {
            *nodes += suffix_nodes;
        }
    }

    if best_hit.is_none() && limits.stop_reason.is_none() {
        limits.stop_reason = Some(match best_distance {
            Some(distance) => {
                format!("not_found:candidates={candidate_count}:best_distance={distance}")
            }
            None => format!("not_found:candidates={candidate_count}"),
        });
    }

    best_hit.map(|moves| (SolveMethod::Macro, moves))
}

fn find_landmark_candidates(
    puzzle: &Puzzle,
    colors: &[Color],
    operations: &[Operation],
    pattern_db: &PatternDb,
    config: &SolverConfig,
    nodes: &mut u64,
    limits: &mut Limits,
) -> Vec<LandmarkCandidate> {
    let initial_score = score_state(
        colors,
        puzzle,
        config.target_mode,
        config.region_pair_weight,
        Some(pattern_db),
    );
    let mut beam = vec![BeamEntry {
        colors: colors.to_vec(),
        path: Vec::new(),
        last_move: None,
        score: initial_score,
        rank_score: initial_score,
    }];
    let mut candidates = Vec::new();
    let mut global_seen = HashSet::new();
    global_seen.insert(state_key(colors));

    for _depth in 0..config.landmark_depth {
        if limits.exceeded(*nodes) {
            break;
        }

        let mut next_entries = Vec::new();
        let mut layer_seen = HashSet::new();

        for entry in &beam {
            for operation in operations {
                if operation.is_raw
                    && entry
                        .last_move
                        .is_some_and(|last| puzzle.moves[operation.moves[0]].inverse == last)
                {
                    continue;
                }

                let mut next_colors = entry.colors.clone();
                for &move_index in &operation.moves {
                    puzzle.apply_move(&mut next_colors, move_index);
                }

                *nodes += 1;
                if limits.exceeded(*nodes) {
                    break;
                }

                let key = state_key(&next_colors);
                if !global_seen.insert(key) || !layer_seen.insert(key) {
                    continue;
                }

                let mut path = entry.path.clone();
                path.extend(&operation.path);
                let score = score_state(
                    &next_colors,
                    puzzle,
                    config.target_mode,
                    config.region_pair_weight,
                    Some(pattern_db),
                ) + path.len() as i32 * config.path_penalty;

                let pattern_distance = pattern_db
                    .distance(&next_colors, puzzle)
                    .unwrap_or_else(|| (pattern_db.depth + 1).min(u8::MAX as usize) as u8);
                keep_best_landmark(
                    &mut candidates,
                    LandmarkCandidate {
                        colors: next_colors.clone(),
                        path: path.clone(),
                        pattern_distance,
                        score,
                    },
                    config.landmark_candidates,
                );

                let rank_score =
                    beam_rank_score(puzzle, &next_colors, &path, score, Some(pattern_db), config);
                next_entries.push(BeamEntry {
                    colors: next_colors,
                    path,
                    last_move: Some(operation.last_move),
                    score,
                    rank_score,
                });
            }
        }

        if next_entries.is_empty() {
            break;
        }

        next_entries.sort_unstable_by(|left, right| {
            left.rank_score
                .cmp(&right.rank_score)
                .then_with(|| left.path.len().cmp(&right.path.len()))
                .then_with(|| left.score.cmp(&right.score))
        });
        next_entries.truncate(config.landmark_width);
        beam = next_entries;
    }

    candidates.sort_unstable_by(landmark_cmp);
    candidates
}

fn keep_best_landmark(
    candidates: &mut Vec<LandmarkCandidate>,
    candidate: LandmarkCandidate,
    limit: usize,
) {
    if limit == 0 {
        return;
    }

    candidates.push(candidate);
    candidates.sort_unstable_by(landmark_cmp);
    candidates.truncate(limit);
}

fn landmark_cmp(left: &LandmarkCandidate, right: &LandmarkCandidate) -> Ordering {
    left.pattern_distance
        .cmp(&right.pattern_distance)
        .then_with(|| left.score.cmp(&right.score))
        .then_with(|| left.path.len().cmp(&right.path.len()))
}

fn build_pattern_db(
    puzzle: &Puzzle,
    config: &SolverConfig,
    nodes: &mut u64,
    limits: &mut Limits,
) -> Option<PatternDb> {
    if !should_build_pattern_db(puzzle, config) {
        return None;
    }

    let mut distances = PatternDbDistances::new(config.pattern_db_projection);
    let mut seen = HashSet::new();
    let mut frontier = Vec::new();

    for seed in reverse_table_seeds(puzzle, config.target_mode) {
        let key = state_key(&seed);
        if seen.insert(key) {
            distances.record(puzzle, &seed, config.pattern_db_projection, 0);
            frontier.push(PatternEntry {
                colors: seed,
                last_move: None,
            });
        }
    }

    for depth in 0..config.pattern_db_depth {
        let mut next = Vec::new();
        let next_distance = (depth + 1).min(u8::MAX as usize) as u8;
        for entry in frontier {
            for move_index in 0..puzzle.moves.len() {
                if is_immediate_inverse(puzzle, entry.last_move, move_index) {
                    continue;
                }

                let mut colors = entry.colors.clone();
                puzzle.apply_move(&mut colors, move_index);
                *nodes += 1;
                if limits.exceeded(*nodes) {
                    return None;
                }

                let key = state_key(&colors);
                if !seen.insert(key) {
                    continue;
                }

                distances.record(puzzle, &colors, config.pattern_db_projection, next_distance);
                next.push(PatternEntry {
                    colors,
                    last_move: Some(move_index),
                });
            }
        }
        frontier = next;
        if frontier.is_empty() {
            break;
        }
    }

    Some(PatternDb {
        distances,
        projection: config.pattern_db_projection,
        use_canonical_fallback: puzzle.layout == LayoutId::E
            && config.pattern_db_projection == ProjectionKind::FaceHistogram
            && config.target_mode != TargetMode::AndroidMultiGoal,
        depth: config.pattern_db_depth,
        weight: config.pattern_db_weight,
    })
}

fn should_build_pattern_db(puzzle: &Puzzle, config: &SolverConfig) -> bool {
    config.pattern_db_enabled
        && config.pattern_db_depth > 0
        && config.pattern_db_weight != 0
        && match puzzle.layout {
            LayoutId::E => {
                puzzle.difficulty == Difficulty::Classic
                    || (puzzle.difficulty == Difficulty::Moderate
                        && config.beam_rank.needs_pattern_db())
            }
            LayoutId::F => {
                puzzle.difficulty == Difficulty::Classic
                    && (config.beam_rank.needs_pattern_db()
                        || should_try_f_pattern_db_portfolio(puzzle, config))
            }
            _ => false,
        }
}

fn build_reverse_table(
    puzzle: &Puzzle,
    config: &SolverConfig,
    nodes: &mut u64,
    limits: &mut Limits,
) -> Option<HashMap<Key, PathBits>> {
    let seeds = reverse_table_seeds(puzzle, config.target_mode);
    let mut states: HashMap<Key, PathBits> = HashMap::new();
    let mut frontier = Vec::new();

    for seed in seeds {
        let key = state_key(&seed);
        if states.insert(key, PathBits::default()).is_none() {
            frontier.push(TableEntry {
                colors: seed,
                path: PathBits::default(),
                last_move: None,
            });
        }
    }

    for _ in 0..config.table_depth_for(puzzle.layout) {
        let mut next = Vec::new();
        for entry in frontier {
            for move_index in 0..puzzle.moves.len() {
                if is_immediate_inverse(puzzle, entry.last_move, move_index) {
                    continue;
                }
                let mut colors = entry.colors.clone();
                puzzle.apply_move(&mut colors, move_index);
                *nodes += 1;
                if limits.exceeded(*nodes) {
                    return None;
                }

                let key = state_key(&colors);
                if states.contains_key(&key) {
                    continue;
                }

                let inverse = puzzle.inverse_index(move_index);
                let path = entry.path.prepend(inverse);
                states.insert(key, path);
                next.push(TableEntry {
                    colors,
                    path,
                    last_move: Some(move_index),
                });
            }
        }
        frontier = next;
        if frontier.is_empty() {
            break;
        }
    }

    Some(states)
}

fn meet_in_middle(
    puzzle: &Puzzle,
    colors: &[Color],
    table: &HashMap<Key, PathBits>,
    config: &SolverConfig,
    nodes: &mut u64,
    limits: &mut Limits,
) -> Option<Vec<MoveIndex>> {
    let start_key = state_key(colors);
    if let Some(path) = table.get(&start_key) {
        return Some(path.to_vec());
    }

    let mut best_hit: Option<Vec<MoveIndex>> = None;
    let mut seen = HashSet::new();
    seen.insert(start_key);
    let mut frontier = vec![ExactEntry {
        colors: colors.to_vec(),
        path: PathBits::default(),
        last_move: None,
    }];

    for _ in 0..config.forward_depth_for(puzzle.layout) {
        let mut next = Vec::new();
        for entry in frontier {
            for move_index in 0..puzzle.moves.len() {
                if is_immediate_inverse(puzzle, entry.last_move, move_index) {
                    continue;
                }

                let mut colors = entry.colors.clone();
                puzzle.apply_move(&mut colors, move_index);
                *nodes += 1;
                if limits.exceeded(*nodes) {
                    return None;
                }

                let key = state_key(&colors);
                if seen.contains(&key) {
                    continue;
                }

                let path = entry.path.append(move_index);
                if let Some(suffix) = table.get(&key) {
                    let mut moves = path.to_vec();
                    moves.extend(suffix.to_vec());
                    keep_shortest(&mut best_hit, moves);
                    continue;
                }

                seen.insert(key);
                next.push(ExactEntry {
                    colors,
                    path,
                    last_move: Some(move_index),
                });
            }
        }
        frontier = next;
        if frontier.is_empty() {
            break;
        }
    }

    best_hit
}

fn beam_rank_score(
    puzzle: &Puzzle,
    colors: &[Color],
    path: &[MoveIndex],
    score: i32,
    pattern_db: Option<&PatternDb>,
    config: &SolverConfig,
) -> i32 {
    beam_rank_score_with_len(puzzle, colors, path.len(), score, pattern_db, config)
}

fn beam_rank_score_with_len(
    puzzle: &Puzzle,
    colors: &[Color],
    path_len: usize,
    score: i32,
    pattern_db: Option<&PatternDb>,
    config: &SolverConfig,
) -> i32 {
    match (config.beam_rank, pattern_db) {
        (BeamRankMode::PatternDistance, Some(pattern_db)) => {
            let distance = pattern_db
                .distance(colors, puzzle)
                .map(i32::from)
                .unwrap_or_else(|| pattern_db.depth as i32 + 1);
            distance * 100_000 + path_len as i32 * config.path_penalty + score / 10
        }
        (BeamRankMode::PatternHybrid, Some(pattern_db)) => {
            let distance = pattern_db
                .distance(colors, puzzle)
                .map(i32::from)
                .unwrap_or_else(|| pattern_db.depth as i32 + 1);
            distance * 50_000 + score / 2 + path_len as i32 * config.path_penalty
        }
        (BeamRankMode::RingRotation, _) => {
            ring_rotation_miss_total(puzzle, colors) as i32 * 10_000
                + path_len as i32 * config.path_penalty
                + score / 10
        }
        (BeamRankMode::RingHybrid, _) => {
            ring_rotation_miss_total(puzzle, colors) as i32 * 4_000
                + score / 2
                + path_len as i32 * config.path_penalty
        }
        _ => score,
    }
}

fn select_corridor_diverse_by_path<T, F>(
    candidates: Vec<T>,
    width: usize,
    corridor_diversity_enabled: bool,
    corridor_prefix_len: usize,
    corridor_quota_percent: usize,
    path_for: F,
) -> Vec<T>
where
    T: Clone,
    F: Fn(&T) -> &[MoveIndex],
{
    if !corridor_diversity_enabled || width == 0 || corridor_quota_percent == 0 {
        return candidates.into_iter().take(width).collect();
    }

    let quota = width
        .saturating_mul(corridor_quota_percent)
        .saturating_add(99)
        / 100;
    let quota = quota.min(width);
    if quota == 0 {
        return candidates.into_iter().take(width).collect();
    }

    let mut selected = Vec::with_capacity(width.min(candidates.len()));
    let mut selected_indexes = vec![false; candidates.len()];
    let mut seen_corridors: HashSet<Vec<MoveIndex>> = HashSet::new();

    for (index, entry) in candidates.iter().enumerate() {
        if selected.len() >= quota {
            break;
        }
        let path = path_for(entry);
        if path.is_empty() {
            continue;
        }
        let signature_len = path.len().min(corridor_prefix_len);
        let signature = path[..signature_len].to_vec();
        if seen_corridors.insert(signature) {
            selected.push(entry.clone());
            selected_indexes[index] = true;
        }
    }

    for (index, entry) in candidates.into_iter().enumerate() {
        if selected.len() >= width {
            break;
        }
        if !selected_indexes[index] {
            selected.push(entry);
        }
    }

    selected
}

fn select_beam_survivors(
    candidates: Vec<BeamEntry>,
    width: usize,
    corridor_diversity_enabled: bool,
    corridor_prefix_len: usize,
    corridor_quota_percent: usize,
) -> Vec<BeamEntry> {
    select_corridor_diverse_by_path(
        candidates,
        width,
        corridor_diversity_enabled,
        corridor_prefix_len,
        corridor_quota_percent,
        |entry| &entry.path,
    )
}

fn macro_beam(
    puzzle: &Puzzle,
    colors: &[Color],
    table: &HashMap<Key, PathBits>,
    operations: &[Operation],
    pattern_db: Option<&PatternDb>,
    tier: Tier,
    config: &SolverConfig,
    nodes: &mut u64,
    limits: &mut Limits,
    first_table_hit: &mut Option<TableHitTelemetry>,
) -> Option<Vec<MoveIndex>> {
    let seed_base = hash_colors(colors);
    let mut best_hit: Option<Vec<MoveIndex>> = None;
    let mut first_hit_restart: Option<usize> = None;

    for restart in 0..tier.restarts {
        let mut rng =
            Mulberry32::new(seed_base.wrapping_add((restart as u32).wrapping_mul(0x9e37_79b9)));
        let jitter = if restart == 0 {
            0
        } else {
            120 + restart as i32 * 60
        };
        let initial_score = score_state(
            colors,
            puzzle,
            config.target_mode,
            config.region_pair_weight,
            pattern_db,
        );
        let mut beam = vec![BeamEntry {
            colors: colors.to_vec(),
            path: Vec::new(),
            last_move: None,
            score: initial_score,
            rank_score: initial_score,
        }];
        let mut first_hit_depth: Option<usize> = None;
        let mut restart_hit_ready = false;

        for depth in 0..tier.max_depth {
            let mut candidates = Vec::new();
            let mut layer_seen = HashSet::new();

            for entry in &beam {
                for operation in operations {
                    if operation.is_raw
                        && entry
                            .last_move
                            .is_some_and(|last| puzzle.moves[operation.moves[0]].inverse == last)
                    {
                        continue;
                    }

                    let mut next_colors = entry.colors.clone();
                    for &move_index in &operation.moves {
                        puzzle.apply_move(&mut next_colors, move_index);
                    }
                    *nodes += 1;
                    if limits.exceeded(*nodes) {
                        return None;
                    }

                    let key = state_key(&next_colors);
                    if !layer_seen.insert(key) {
                        continue;
                    }

                    let mut path = entry.path.clone();
                    path.extend(&operation.path);

                    if let Some(suffix) = table.get(&key) {
                        let prefix_len = path.len();
                        let suffix_vec = suffix.to_vec();
                        let suffix_len = suffix_vec.len();
                        if first_table_hit.is_none() {
                            *first_table_hit = Some(TableHitTelemetry {
                                target: config.target_mode,
                                operation_profile: config.operation_profile,
                                beam_rank: config.beam_rank,
                                pattern_db: pattern_db.is_some(),
                                depth: depth + 1,
                                restart,
                                nodes: *nodes,
                                elapsed_ms: limits.started.elapsed().as_millis(),
                                prefix_len,
                                suffix_len,
                                total_len: prefix_len + suffix_len,
                            });
                        }
                        path.extend(suffix_vec);
                        if config.retrograde_suffix_beam_first_hit {
                            return Some(path);
                        }
                        keep_shortest(&mut best_hit, path);
                        first_hit_depth.get_or_insert(depth);
                        first_hit_restart.get_or_insert(restart);
                        continue;
                    }

                    if is_target_solved(&next_colors, puzzle, config.target_mode) {
                        keep_shortest(&mut best_hit, path);
                        first_hit_depth.get_or_insert(depth);
                        first_hit_restart.get_or_insert(restart);
                        continue;
                    }

                    let score = score_state(
                        &next_colors,
                        puzzle,
                        config.target_mode,
                        config.region_pair_weight,
                        pattern_db,
                    ) + path.len() as i32 * config.path_penalty;
                    let rank_score =
                        beam_rank_score(puzzle, &next_colors, &path, score, pattern_db, config)
                            + jitter_sample(&mut rng, jitter);
                    candidates.push(BeamEntry {
                        colors: next_colors,
                        path,
                        last_move: Some(operation.last_move),
                        score,
                        rank_score,
                    });
                }
            }

            if best_hit.is_some()
                && first_hit_depth.is_some_and(|hit_depth| depth >= hit_depth + config.hit_patience)
            {
                restart_hit_ready = true;
                break;
            }

            if candidates.is_empty() {
                break;
            }

            candidates.sort_unstable_by(|left, right| {
                left.rank_score
                    .cmp(&right.rank_score)
                    .then_with(|| left.path.len().cmp(&right.path.len()))
                    .then_with(|| left.score.cmp(&right.score))
            });
            beam = select_beam_survivors(
                candidates,
                tier.width,
                config.corridor_diversity_enabled,
                config.corridor_prefix_len,
                config.corridor_quota_percent,
            );
        }

        if restart_hit_ready
            && first_hit_restart
                .is_some_and(|hit_restart| restart >= hit_restart + config.hit_restart_patience)
        {
            return best_hit;
        }
    }

    best_hit
}

fn find_ring_residue_prefix(
    puzzle: &Puzzle,
    colors: &[Color],
    config: &SolverConfig,
    artifacts: &SolverArtifacts,
    targets: &[Vec<Color>],
    gate: usize,
) -> ResiduePrefixResult {
    let started = Instant::now();
    let initial_mismatches = best_mismatches_to_targets(colors, targets);
    if initial_mismatches <= gate {
        return ResiduePrefixResult {
            found: true,
            reason: "initial_within_gate".to_string(),
            rank_label: "none".to_string(),
            profile_label: "none".to_string(),
            colors: colors.to_vec(),
            moves: Vec::new(),
            mismatches: initial_mismatches,
            nodes: 0,
            elapsed_ms: started.elapsed().as_millis(),
        };
    }

    let mut best_failure = ResiduePrefixResult {
        found: false,
        reason: "not_found".to_string(),
        rank_label: "none".to_string(),
        profile_label: "none".to_string(),
        colors: colors.to_vec(),
        moves: Vec::new(),
        mismatches: initial_mismatches,
        nodes: 0,
        elapsed_ms: 0,
    };

    for variant in &artifacts.variants {
        for operation_set in &variant.operation_sets {
            for rank_mode in beam_rank_execution_modes(config.beam_rank) {
                let mut variant_config = config.clone();
                variant_config.target_mode = variant.target_mode;
                variant_config.operation_profile = operation_set.profile;
                variant_config.time_limit_ms = operation_set.time_limit_ms;
                variant_config.beam_rank = rank_mode;
                let mut limits = Limits::new(config.max_nodes, operation_set.time_limit_ms);
                let mut nodes = 0u64;
                let attempt = residue_prefix_beam(
                    puzzle,
                    colors,
                    targets,
                    &operation_set.operations,
                    None,
                    gate,
                    &variant_config,
                    &mut nodes,
                    &mut limits,
                );

                match attempt {
                    Some(mut result) => {
                        result.rank_label = rank_mode.label().to_string();
                        result.profile_label = operation_set.profile.label().to_string();
                        result.nodes = nodes;
                        result.elapsed_ms = started.elapsed().as_millis();
                        return result;
                    }
                    None => {
                        let reason = limits
                            .stop_reason
                            .clone()
                            .unwrap_or_else(|| "not_found".to_string());
                        if best_failure.mismatches > gate {
                            best_failure.reason = format!(
                                "{}:{}:{}",
                                operation_set.profile.label(),
                                rank_mode.label(),
                                reason
                            );
                            best_failure.rank_label = rank_mode.label().to_string();
                            best_failure.profile_label = operation_set.profile.label().to_string();
                            best_failure.nodes += nodes;
                        }
                    }
                }
            }
        }
    }

    best_failure.elapsed_ms = started.elapsed().as_millis();
    best_failure
}

fn residue_prefix_beam(
    puzzle: &Puzzle,
    colors: &[Color],
    targets: &[Vec<Color>],
    operations: &[Operation],
    pattern_db: Option<&PatternDb>,
    gate: usize,
    config: &SolverConfig,
    nodes: &mut u64,
    limits: &mut Limits,
) -> Option<ResiduePrefixResult> {
    let seed_base = hash_colors(colors);

    for restart in 0..config
        .tiers
        .iter()
        .map(|tier| tier.restarts)
        .max()
        .unwrap_or(1)
    {
        let tier = config.tiers.first().copied().unwrap_or(Tier {
            max_depth: 80,
            width: 3_000,
            restarts: 1,
        });
        if restart >= tier.restarts {
            break;
        }
        let mut rng =
            Mulberry32::new(seed_base.wrapping_add((restart as u32).wrapping_mul(0x9e37_79b9)));
        let jitter = if restart == 0 {
            0
        } else {
            120 + restart as i32 * 60
        };
        let initial_score = score_state(
            colors,
            puzzle,
            config.target_mode,
            config.region_pair_weight,
            pattern_db,
        );
        let mut beam = vec![BeamEntry {
            colors: colors.to_vec(),
            path: Vec::new(),
            last_move: None,
            score: initial_score,
            rank_score: initial_score,
        }];

        for _depth in 0..tier.max_depth {
            let mut candidates = Vec::new();
            let mut layer_seen = HashSet::new();

            for entry in &beam {
                for operation in operations {
                    if operation.is_raw
                        && entry
                            .last_move
                            .is_some_and(|last| puzzle.moves[operation.moves[0]].inverse == last)
                    {
                        continue;
                    }

                    let mut next_colors = entry.colors.clone();
                    for &move_index in &operation.moves {
                        puzzle.apply_move(&mut next_colors, move_index);
                    }
                    *nodes += 1;
                    if limits.exceeded(*nodes) {
                        return None;
                    }

                    let key = state_key(&next_colors);
                    if !layer_seen.insert(key) {
                        continue;
                    }

                    let mut path = entry.path.clone();
                    path.extend(&operation.path);
                    let mismatches = best_mismatches_to_targets(&next_colors, targets);
                    if mismatches <= gate {
                        return Some(ResiduePrefixResult {
                            found: true,
                            reason: "found_gate".to_string(),
                            rank_label: config.beam_rank.label().to_string(),
                            profile_label: config.operation_profile.label().to_string(),
                            colors: next_colors,
                            moves: path,
                            mismatches,
                            nodes: *nodes,
                            elapsed_ms: 0,
                        });
                    }

                    let score = score_state(
                        &next_colors,
                        puzzle,
                        config.target_mode,
                        config.region_pair_weight,
                        pattern_db,
                    ) + path.len() as i32 * config.path_penalty;
                    let rank_score =
                        beam_rank_score(puzzle, &next_colors, &path, score, pattern_db, config)
                            + jitter_sample(&mut rng, jitter);
                    candidates.push(BeamEntry {
                        colors: next_colors,
                        path,
                        last_move: Some(operation.last_move),
                        score,
                        rank_score,
                    });
                }
            }

            if candidates.is_empty() {
                break;
            }
            candidates.sort_unstable_by(|left, right| {
                left.rank_score
                    .cmp(&right.rank_score)
                    .then_with(|| left.path.len().cmp(&right.path.len()))
                    .then_with(|| left.score.cmp(&right.score))
            });
            beam = select_beam_survivors(
                candidates,
                tier.width,
                config.corridor_diversity_enabled,
                config.corridor_prefix_len,
                config.corridor_quota_percent,
            );
        }
    }

    None
}

fn collect_pair_region_prefixes(
    puzzle: &Puzzle,
    colors: &[Color],
    config: &SolverConfig,
    table: &HashMap<Key, PathBits>,
    operations: &[Operation],
    pattern_db: Option<&PatternDb>,
    max_prefixes: usize,
    nodes: &mut u64,
    limits: &mut Limits,
) -> Vec<Vec<MoveIndex>> {
    let max_prefixes = max_prefixes.max(1);
    let mut prefixes = Vec::new();
    let mut seen_prefixes = HashSet::new();

    if is_target_solved(colors, puzzle, config.target_mode) {
        record_prefix(&mut prefixes, &mut seen_prefixes, Vec::new(), max_prefixes);
        return prefixes;
    }

    if let Some(moves) = meet_in_middle(puzzle, colors, table, config, nodes, limits) {
        record_prefix(&mut prefixes, &mut seen_prefixes, moves, max_prefixes);
    }

    for &tier in &config.tiers {
        if prefixes.len() >= max_prefixes || limits.exceeded(*nodes) {
            break;
        }

        collect_macro_prefixes(
            puzzle,
            colors,
            table,
            operations,
            pattern_db,
            tier,
            config,
            max_prefixes,
            &mut prefixes,
            &mut seen_prefixes,
            nodes,
            limits,
        );
    }

    prefixes
        .sort_unstable_by(|left, right| left.len().cmp(&right.len()).then_with(|| left.cmp(right)));
    prefixes.truncate(max_prefixes);
    prefixes
}

fn collect_macro_prefixes(
    puzzle: &Puzzle,
    colors: &[Color],
    table: &HashMap<Key, PathBits>,
    operations: &[Operation],
    pattern_db: Option<&PatternDb>,
    tier: Tier,
    config: &SolverConfig,
    max_prefixes: usize,
    prefixes: &mut Vec<Vec<MoveIndex>>,
    seen_prefixes: &mut HashSet<Vec<MoveIndex>>,
    nodes: &mut u64,
    limits: &mut Limits,
) {
    let seed_base = hash_colors(colors);

    for restart in 0..tier.restarts {
        if prefixes.len() >= max_prefixes || limits.exceeded(*nodes) {
            return;
        }

        let mut rng =
            Mulberry32::new(seed_base.wrapping_add((restart as u32).wrapping_mul(0x9e37_79b9)));
        let jitter = if restart == 0 {
            0
        } else {
            120 + restart as i32 * 60
        };
        let initial_score = score_state(
            colors,
            puzzle,
            config.target_mode,
            config.region_pair_weight,
            pattern_db,
        );
        let mut beam = vec![BeamEntry {
            colors: colors.to_vec(),
            path: Vec::new(),
            last_move: None,
            score: initial_score,
            rank_score: initial_score,
        }];

        for _ in 0..tier.max_depth {
            if prefixes.len() >= max_prefixes || limits.exceeded(*nodes) {
                return;
            }

            let mut candidates = Vec::new();
            let mut layer_seen = HashSet::new();

            for entry in &beam {
                for operation in operations {
                    if operation.is_raw
                        && entry
                            .last_move
                            .is_some_and(|last| puzzle.moves[operation.moves[0]].inverse == last)
                    {
                        continue;
                    }

                    let mut next_colors = entry.colors.clone();
                    for &move_index in &operation.moves {
                        puzzle.apply_move(&mut next_colors, move_index);
                    }
                    *nodes += 1;
                    if limits.exceeded(*nodes) {
                        return;
                    }

                    let key = state_key(&next_colors);
                    if !layer_seen.insert(key) {
                        continue;
                    }

                    let mut path = entry.path.clone();
                    path.extend(&operation.path);

                    if let Some(suffix) = table.get(&key) {
                        path.extend(suffix.to_vec());
                        if record_prefix(prefixes, seen_prefixes, path, max_prefixes) {
                            return;
                        }
                        continue;
                    }

                    if is_target_solved(&next_colors, puzzle, config.target_mode) {
                        if record_prefix(prefixes, seen_prefixes, path, max_prefixes) {
                            return;
                        }
                        continue;
                    }

                    let score = score_state(
                        &next_colors,
                        puzzle,
                        config.target_mode,
                        config.region_pair_weight,
                        pattern_db,
                    ) + path.len() as i32 * config.path_penalty;
                    let rank_score = score + jitter_sample(&mut rng, jitter);
                    candidates.push(BeamEntry {
                        colors: next_colors,
                        path,
                        last_move: Some(operation.last_move),
                        score,
                        rank_score,
                    });
                }
            }

            if candidates.is_empty() {
                break;
            }

            candidates.sort_unstable_by(|left, right| {
                left.rank_score
                    .cmp(&right.rank_score)
                    .then_with(|| left.path.len().cmp(&right.path.len()))
                    .then_with(|| left.score.cmp(&right.score))
            });
            candidates.truncate(tier.width);
            beam = candidates;
        }
    }
}

fn record_prefix(
    prefixes: &mut Vec<Vec<MoveIndex>>,
    seen_prefixes: &mut HashSet<Vec<MoveIndex>>,
    prefix: Vec<MoveIndex>,
    max_prefixes: usize,
) -> bool {
    if seen_prefixes.insert(prefix.clone()) {
        prefixes.push(prefix);
    }
    prefixes.len() >= max_prefixes
}

fn find_phase_prefixes(
    puzzle: &Puzzle,
    colors: &[Color],
    spec: &PhaseSpec,
    operations: &[Operation],
    tier: Tier,
    max_prefixes: usize,
    path_penalty: i32,
    nodes: &mut u64,
    limits: &mut Limits,
) -> Vec<Vec<MoveIndex>> {
    let max_prefixes = max_prefixes.max(1);
    let mut prefixes = Vec::new();
    let mut seen_prefixes = HashSet::new();

    if phase_target_solved(puzzle, colors, spec) {
        record_prefix(&mut prefixes, &mut seen_prefixes, Vec::new(), max_prefixes);
        return prefixes;
    }

    let seed_base = hash_colors(colors) ^ hash_phase_spec(spec);
    for restart in 0..tier.restarts {
        if prefixes.len() >= max_prefixes || limits.exceeded(*nodes) {
            break;
        }

        let mut rng =
            Mulberry32::new(seed_base.wrapping_add((restart as u32).wrapping_mul(0x9e37_79b9)));
        let jitter = if restart == 0 {
            0
        } else {
            100 + restart as i32 * 70
        };
        let initial_score = phase_score(puzzle, colors, spec, 0, path_penalty);
        let mut beam = vec![BeamEntry {
            colors: colors.to_vec(),
            path: Vec::new(),
            last_move: None,
            score: initial_score,
            rank_score: initial_score,
        }];
        let mut global_seen = HashSet::new();
        global_seen.insert(state_key(colors));

        for _depth in 0..tier.max_depth {
            if prefixes.len() >= max_prefixes || limits.exceeded(*nodes) {
                break;
            }

            let mut candidates = Vec::new();
            let mut layer_seen = HashSet::new();
            for entry in &beam {
                for operation in operations {
                    if operation.is_raw
                        && entry
                            .last_move
                            .is_some_and(|last| puzzle.moves[operation.moves[0]].inverse == last)
                    {
                        continue;
                    }

                    let mut next_colors = entry.colors.clone();
                    for &move_index in &operation.moves {
                        puzzle.apply_move(&mut next_colors, move_index);
                    }
                    *nodes += 1;
                    if limits.exceeded(*nodes) {
                        break;
                    }

                    let key = state_key(&next_colors);
                    if !global_seen.insert(key) || !layer_seen.insert(key) {
                        continue;
                    }

                    let mut path = entry.path.clone();
                    path.extend(&operation.path);
                    if phase_target_solved(puzzle, &next_colors, spec) {
                        if record_prefix(&mut prefixes, &mut seen_prefixes, path, max_prefixes) {
                            break;
                        }
                        continue;
                    }

                    let score = phase_score(puzzle, &next_colors, spec, path.len(), path_penalty);
                    let rank_score = score + jitter_sample(&mut rng, jitter);
                    candidates.push(BeamEntry {
                        colors: next_colors,
                        path,
                        last_move: Some(operation.last_move),
                        score,
                        rank_score,
                    });
                }
            }

            if candidates.is_empty() {
                break;
            }

            candidates.sort_unstable_by(|left, right| {
                left.rank_score
                    .cmp(&right.rank_score)
                    .then_with(|| left.path.len().cmp(&right.path.len()))
                    .then_with(|| left.score.cmp(&right.score))
            });
            candidates.truncate(tier.width);
            beam = candidates;
        }
    }

    prefixes
        .sort_unstable_by(|left, right| left.len().cmp(&right.len()).then_with(|| left.cmp(right)));
    prefixes.truncate(max_prefixes);
    prefixes
}

fn moved_sticker_position(puzzle: &Puzzle, position: usize, move_index: MoveIndex) -> usize {
    let mv = &puzzle.moves[move_index];
    let Some(cycle_index) = mv.cycle.iter().position(|&index| index == position) else {
        return position;
    };
    let next_index = if mv.direction > 0 {
        (cycle_index + 1) % mv.cycle.len()
    } else {
        (cycle_index + mv.cycle.len() - 1) % mv.cycle.len()
    };
    mv.cycle[next_index]
}

fn operation_avoids_positions(puzzle: &Puzzle, operation: &Operation, positions: &[usize]) -> bool {
    operation.moves.iter().all(|&move_index| {
        let cycle = &puzzle.moves[move_index].cycle;
        positions.iter().all(|position| !cycle.contains(position))
    })
}

fn build_corner_stage_operations(
    puzzle: &Puzzle,
    plan: &CornerPlan,
    operations: &[Operation],
    include_shielded_operations: bool,
    shielded_body_depth: usize,
) -> Vec<Operation> {
    let mut preserving = operations
        .iter()
        .filter(|operation| {
            operation
                .moves
                .iter()
                .chain(operation.path.iter())
                .all(|&move_index| {
                    let mv = &puzzle.moves[move_index];
                    !plan.forbidden_tapes.contains(&TapeCoord {
                        axis: mv.axis,
                        layer: mv.layer,
                    })
                })
        })
        .cloned()
        .collect::<Vec<_>>();
    if !include_shielded_operations {
        return preserving;
    }

    let mut seen = preserving
        .iter()
        .map(|operation| operation_permutation_key(puzzle, &operation.moves))
        .collect::<HashSet<_>>();
    let protected_positions = plan
        .seed_regions
        .iter()
        .flat_map(|(_, indexes)| indexes.iter().copied())
        .collect::<Vec<_>>();
    let test_operations = build_operations(puzzle, OperationProfile::Raw);

    for &restricted_tape in &plan.forbidden_tapes {
        let Some(positive_move) = puzzle.find_move(restricted_tape.axis, restricted_tape.layer, 1)
        else {
            continue;
        };
        let cycle_len = puzzle.moves[positive_move].cycle.len();
        let shifts = [(1_i8, cycle_len / 2), (-1_i8, (cycle_len - 1) / 2)];
        for (direction, max_steps) in shifts {
            let Some(shift_move) =
                puzzle.find_move(restricted_tape.axis, restricted_tape.layer, direction)
            else {
                continue;
            };
            let inverse_shift = puzzle.inverse_index(shift_move);
            for steps in 1..=max_steps {
                let mut shifted_positions = protected_positions.clone();
                for _ in 0..steps {
                    for position in &mut shifted_positions {
                        *position = moved_sticker_position(puzzle, *position, shift_move);
                    }
                }

                let eligible_test_operations = test_operations
                    .iter()
                    .filter(|operation| {
                        operation_avoids_positions(puzzle, operation, &shifted_positions)
                    })
                    .collect::<Vec<_>>();
                for test_operation in &eligible_test_operations {
                    let mut moves = vec![shift_move; steps];
                    moves.extend(&test_operation.moves);
                    moves.extend(std::iter::repeat_n(inverse_shift, steps));
                    add_macro_operation(puzzle, &mut preserving, &mut seen, moves);
                }
                if shielded_body_depth >= 2 {
                    for first in &eligible_test_operations {
                        for second in &eligible_test_operations {
                            let first_move = &puzzle.moves[first.moves[0]];
                            let second_move = &puzzle.moves[second.moves[0]];
                            if first_move.axis == second_move.axis {
                                continue;
                            }
                            let mut moves = vec![shift_move; steps];
                            moves.extend(&first.moves);
                            moves.extend(&second.moves);
                            moves.extend(std::iter::repeat_n(inverse_shift, steps));
                            add_macro_operation(puzzle, &mut preserving, &mut seen, moves);
                        }
                    }
                }
            }
        }
    }

    preserving
}

fn find_protected_corner_prefixes(
    puzzle: &Puzzle,
    colors: &[Color],
    spec: &PhaseSpec,
    operations: &[Operation],
    tier: Tier,
    max_prefixes: usize,
    path_penalty: i32,
    include_shielded_operations: bool,
    shielded_body_depth: usize,
    seed_branches: usize,
    arm_branches: usize,
    nodes: &mut u64,
    limits: &mut Limits,
) -> Vec<Vec<MoveIndex>> {
    let Some(plan) = &spec.corner_plan else {
        return Vec::new();
    };

    let mut seed_spec = spec.clone();
    seed_spec.near_misses_per_face = 0;
    if let Some(seed_plan) = &mut seed_spec.corner_plan {
        seed_plan.stage = CornerStage::Seed;
    }
    let seed_tier = Tier {
        max_depth: tier.max_depth.min(12),
        width: tier.width.clamp(1_500, 4_000),
        restarts: tier.restarts,
    };
    let seed_operations = build_operations(puzzle, OperationProfile::Raw);
    let seed_prefixes = find_phase_prefixes(
        puzzle,
        colors,
        &seed_spec,
        &seed_operations,
        seed_tier,
        seed_branches,
        path_penalty,
        nodes,
        limits,
    );
    if seed_prefixes.is_empty() {
        let reason = limits
            .stop_reason
            .take()
            .unwrap_or_else(|| "not_found".to_string());
        limits.stop_reason = Some(format!("corner_seed_{reason}"));
        return Vec::new();
    }

    let allowed_operations = build_corner_stage_operations(
        puzzle,
        plan,
        operations,
        include_shielded_operations,
        shielded_body_depth,
    );
    if allowed_operations.is_empty() {
        limits.stop_reason = Some("corner_no_allowed_operations".to_string());
        return Vec::new();
    }

    let mut prefixes = Vec::new();
    let mut seen_prefixes = HashSet::new();
    let mut arm_spec = spec.clone();
    if let Some(arm_plan) = &mut arm_spec.corner_plan {
        arm_plan.stage = CornerStage::Arms;
    }
    let mut arm_hits = 0usize;
    for seed_prefix in seed_prefixes {
        if prefixes.len() >= max_prefixes || limits.exceeded(*nodes) {
            break;
        }
        let mut seeded_colors = colors.to_vec();
        puzzle.apply_moves(&mut seeded_colors, &seed_prefix);
        let arm_prefixes = find_phase_prefixes(
            puzzle,
            &seeded_colors,
            &arm_spec,
            &allowed_operations,
            tier,
            if plan.stage == CornerStage::Arms {
                max_prefixes
            } else {
                arm_branches
            },
            path_penalty,
            nodes,
            limits,
        );
        arm_hits += arm_prefixes.len();

        for arm_prefix in arm_prefixes {
            let mut seed_and_arm = seed_prefix.clone();
            seed_and_arm.extend(&arm_prefix);
            if plan.stage == CornerStage::Arms {
                if record_prefix(
                    &mut prefixes,
                    &mut seen_prefixes,
                    seed_and_arm,
                    max_prefixes,
                ) {
                    break;
                }
                continue;
            }

            if limits.exceeded(*nodes) {
                break;
            }
            let mut armed_colors = seeded_colors.clone();
            puzzle.apply_moves(&mut armed_colors, &arm_prefix);
            let lineage_capacity = seed_branches.saturating_mul(arm_branches).max(1);
            let block_prefixes_per_lineage =
                max_prefixes.saturating_add(lineage_capacity - 1) / lineage_capacity;
            let block_prefixes = find_phase_prefixes(
                puzzle,
                &armed_colors,
                spec,
                &allowed_operations,
                tier,
                block_prefixes_per_lineage,
                path_penalty,
                nodes,
                limits,
            );
            for block_prefix in block_prefixes {
                let mut prefix = seed_and_arm.clone();
                prefix.extend(block_prefix);
                if record_prefix(&mut prefixes, &mut seen_prefixes, prefix, max_prefixes) {
                    break;
                }
            }
        }
    }
    prefixes
        .sort_unstable_by(|left, right| left.len().cmp(&right.len()).then_with(|| left.cmp(right)));
    prefixes.truncate(max_prefixes);
    if prefixes.is_empty() {
        let reason = limits
            .stop_reason
            .take()
            .unwrap_or_else(|| "not_found".to_string());
        let stage = if arm_hits == 0 {
            "corner_arms"
        } else {
            "corner_block"
        };
        limits.stop_reason = Some(format!(
            "{stage}_{reason}_allowed_ops={}",
            allowed_operations.len()
        ));
    }
    prefixes
}

fn hash_phase_spec(spec: &PhaseSpec) -> u32 {
    let mut hash = spec.kind as u32 + 1;
    for &face in &spec.faces {
        hash = hash.wrapping_mul(31).wrapping_add(face.index() as u32 + 1);
    }
    for tape in &spec.tapes {
        hash = hash
            .wrapping_mul(31)
            .wrapping_add(tape.axis as u32 * 8 + tape.layer as u32 + 1);
    }
    if let Some(plan) = &spec.corner_plan {
        hash = hash.wrapping_mul(31).wrapping_add(match plan.stage {
            CornerStage::Seed => 1,
            CornerStage::Arms => 2,
            CornerStage::Block => 3,
        });
        for tape in &plan.forbidden_tapes {
            hash = hash
                .wrapping_mul(31)
                .wrapping_add(tape.axis as u32 * 8 + tape.layer as u32 + 1);
        }
    }
    hash = hash
        .wrapping_mul(31)
        .wrapping_add(spec.near_misses_per_face as u32 + 1);
    hash
}

#[derive(Debug)]
struct RankedPhasePrefix {
    prefix: Vec<MoveIndex>,
    rank_score: i32,
    len: usize,
}

fn dedupe_phase_prefix_states(
    puzzle: &Puzzle,
    colors: &[Color],
    prefixes: &mut Vec<Vec<MoveIndex>>,
) {
    let mut best_by_state: HashMap<Key, Vec<MoveIndex>> = HashMap::new();
    for prefix in prefixes.drain(..) {
        let mut intermediate_colors = colors.to_vec();
        puzzle.apply_moves(&mut intermediate_colors, &prefix);
        let key = state_key(&intermediate_colors);
        match best_by_state.get_mut(&key) {
            Some(current)
                if (current.len(), current.as_slice()) <= (prefix.len(), prefix.as_slice()) => {}
            Some(current) => *current = prefix,
            None => {
                best_by_state.insert(key, prefix);
            }
        }
    }
    prefixes.extend(best_by_state.into_values());
}

fn rank_phase_prefixes(
    puzzle: &Puzzle,
    colors: &[Color],
    spec: &PhaseSpec,
    options: &BenchOptions,
    prefixes: &mut Vec<Vec<MoveIndex>>,
    artifacts: &SolverArtifacts,
    pattern_db: Option<&PatternDb>,
) -> u64 {
    if prefixes.len() <= 1 {
        return 0;
    }

    if matches!(options.phase_prefix_rank, PhasePrefixRank::SuffixProbe) {
        return rank_phase_prefixes_with_suffix_probe(puzzle, colors, options, prefixes, artifacts);
    }

    let target_mode = acceptance_target(options.solver.target_mode);
    let mut ranked = prefixes
        .drain(..)
        .map(|prefix| {
            let mut intermediate_colors = colors.to_vec();
            puzzle.apply_moves(&mut intermediate_colors, &prefix);
            let global_score = score_state(
                &intermediate_colors,
                puzzle,
                target_mode,
                options.solver.region_pair_weight,
                None,
            );
            let phase_score = phase_score(
                puzzle,
                &intermediate_colors,
                spec,
                prefix.len(),
                options.solver.path_penalty,
            );
            let len_penalty = prefix.len() as i32 * options.solver.path_penalty;
            let rank_score = match options.phase_prefix_rank {
                PhasePrefixRank::Length => len_penalty,
                PhasePrefixRank::Global => global_score + len_penalty,
                PhasePrefixRank::Combined => phase_score + global_score,
                PhasePrefixRank::Lookahead => {
                    phase_prefix_lookahead_score(
                        puzzle,
                        &intermediate_colors,
                        target_mode,
                        options.solver.region_pair_weight,
                        None,
                        options.phase_rank_lookahead_depth,
                        options.phase_rank_lookahead_width,
                    ) + len_penalty
                }
                PhasePrefixRank::PatternDistance => {
                    pattern_db
                        .and_then(|db| db.distance(&intermediate_colors, puzzle))
                        .map(|distance| i32::from(distance) * 100_000)
                        .unwrap_or((options.solver.pattern_db_depth as i32 + 1) * 100_000)
                        + len_penalty
                }
                PhasePrefixRank::PatternLookahead => {
                    phase_prefix_lookahead_score(
                        puzzle,
                        &intermediate_colors,
                        target_mode,
                        options.solver.region_pair_weight,
                        pattern_db,
                        options.phase_rank_lookahead_depth,
                        options.phase_rank_lookahead_width,
                    ) + len_penalty
                }
                PhasePrefixRank::SuffixProbe => unreachable!("handled before ranking"),
            };
            RankedPhasePrefix {
                len: prefix.len(),
                prefix,
                rank_score,
            }
        })
        .collect::<Vec<_>>();

    ranked.sort_unstable_by(|left, right| {
        left.rank_score
            .cmp(&right.rank_score)
            .then_with(|| left.len.cmp(&right.len))
            .then_with(|| left.prefix.cmp(&right.prefix))
    });
    prefixes.extend(ranked.into_iter().map(|ranked| ranked.prefix));
    0
}

fn rank_phase_prefixes_with_suffix_probe(
    puzzle: &Puzzle,
    colors: &[Color],
    options: &BenchOptions,
    prefixes: &mut Vec<Vec<MoveIndex>>,
    artifacts: &SolverArtifacts,
) -> u64 {
    let target_mode = acceptance_target(options.solver.target_mode);
    let mut ranked = prefixes
        .drain(..)
        .map(|prefix| {
            let mut intermediate_colors = colors.to_vec();
            puzzle.apply_moves(&mut intermediate_colors, &prefix);
            let len_penalty = prefix.len() as i32 * options.solver.path_penalty;
            let cheap_score = phase_prefix_lookahead_score(
                puzzle,
                &intermediate_colors,
                target_mode,
                options.solver.region_pair_weight,
                None,
                options.phase_rank_lookahead_depth,
                options.phase_rank_lookahead_width,
            ) + len_penalty;
            RankedPhasePrefix {
                len: prefix.len(),
                prefix,
                rank_score: 1_200_000_000 + cheap_score,
            }
        })
        .collect::<Vec<_>>();

    sort_ranked_phase_prefixes(&mut ranked);

    let mut total_nodes = 0u64;
    let probe_count = options
        .phase_prefix_suffix_probe_candidates
        .min(ranked.len());
    for ranked_prefix in ranked.iter_mut().take(probe_count) {
        let mut intermediate_colors = colors.to_vec();
        puzzle.apply_moves(&mut intermediate_colors, &ranked_prefix.prefix);
        let cheap_score = ranked_prefix.rank_score - 1_200_000_000;
        let fallback_score = 1_000_000_000 + cheap_score;
        let (score, nodes) = phase_prefix_suffix_probe_score(
            puzzle,
            colors,
            &intermediate_colors,
            &ranked_prefix.prefix,
            options,
            artifacts,
            fallback_score,
        );
        ranked_prefix.rank_score = score;
        total_nodes += nodes;
    }

    sort_ranked_phase_prefixes(&mut ranked);
    prefixes.extend(ranked.into_iter().map(|ranked| ranked.prefix));
    total_nodes
}

fn sort_ranked_phase_prefixes(ranked: &mut [RankedPhasePrefix]) {
    ranked.sort_unstable_by(|left, right| {
        left.rank_score
            .cmp(&right.rank_score)
            .then_with(|| left.len.cmp(&right.len))
            .then_with(|| left.prefix.cmp(&right.prefix))
    });
}

fn phase_prefix_suffix_probe_score(
    puzzle: &Puzzle,
    original_colors: &[Color],
    intermediate_colors: &[Color],
    prefix: &[MoveIndex],
    options: &BenchOptions,
    artifacts: &SolverArtifacts,
    fallback_score: i32,
) -> (i32, u64) {
    if options.phase_prefix_suffix_probe_time_limit_ms == 0 {
        return (fallback_score, 0);
    }

    let mut probe_config = options.solver.clone();
    probe_config.optimize = false;
    let probe = solve_suffix_from_prefix(
        puzzle,
        original_colors,
        intermediate_colors,
        prefix,
        0,
        &probe_config,
        artifacts,
        options.phase_prefix_suffix_probe_time_limit_ms,
        false,
    );
    let nodes = probe.nodes;
    if !probe.found {
        return (fallback_score, nodes);
    }

    let elapsed_penalty = probe.elapsed_ms.min(99_999) as i32;
    (
        probe.raw_moves.len() as i32 * 100_000 + elapsed_penalty,
        nodes,
    )
}

fn phase_prefix_lookahead_score(
    puzzle: &Puzzle,
    colors: &[Color],
    target_mode: TargetMode,
    region_pair_weight: i32,
    pattern_db: Option<&PatternDb>,
    depth: usize,
    width: usize,
) -> i32 {
    let initial_score = score_state(colors, puzzle, target_mode, region_pair_weight, pattern_db);
    if depth == 0 || width == 0 {
        return initial_score;
    }

    let mut best_score = initial_score;
    let mut beam = vec![(colors.to_vec(), None, initial_score)];
    let mut seen = HashSet::new();
    seen.insert(state_key(colors));

    for _ in 0..depth {
        let mut candidates = Vec::new();
        for (entry_colors, last_move, _) in &beam {
            for move_index in 0..puzzle.moves.len() {
                if is_immediate_inverse(puzzle, *last_move, move_index) {
                    continue;
                }

                let mut next_colors = entry_colors.clone();
                puzzle.apply_move(&mut next_colors, move_index);
                if !seen.insert(state_key(&next_colors)) {
                    continue;
                }
                let score = score_state(
                    &next_colors,
                    puzzle,
                    target_mode,
                    region_pair_weight,
                    pattern_db,
                );
                best_score = best_score.min(score);
                candidates.push((next_colors, Some(move_index), score));
            }
        }

        if candidates.is_empty() {
            break;
        }
        candidates.sort_unstable_by_key(|candidate| candidate.2);
        candidates.truncate(width);
        beam = candidates;
    }

    best_score
}

fn keep_shortest(best: &mut Option<Vec<MoveIndex>>, candidate: Vec<MoveIndex>) {
    match best {
        Some(current) if current.len() <= candidate.len() => {}
        _ => *best = Some(candidate),
    }
}

fn keep_best_result(best: &mut Option<SolveResult>, candidate: SolveResult) {
    match best {
        Some(current) => {
            let candidate_key = (
                candidate.optimized_moves.len(),
                candidate.raw_moves.len(),
                candidate.nodes,
            );
            let current_key = (
                current.optimized_moves.len(),
                current.raw_moves.len(),
                current.nodes,
            );
            if candidate_key < current_key {
                *current = candidate;
            }
        }
        None => *best = Some(candidate),
    }
}

fn target_variants(target_mode: TargetMode) -> Vec<TargetMode> {
    match target_mode {
        TargetMode::AndroidPortfolio => vec![TargetMode::Android, TargetMode::AndroidMultiGoal],
        TargetMode::PairRegion => vec![TargetMode::PairRegion],
        other => vec![other],
    }
}

fn acceptance_target(target_mode: TargetMode) -> TargetMode {
    match target_mode {
        TargetMode::Uniform => TargetMode::Uniform,
        TargetMode::PairRegion => TargetMode::PairRegion,
        TargetMode::Android | TargetMode::AndroidMultiGoal | TargetMode::AndroidPortfolio => {
            TargetMode::Android
        }
    }
}

fn solution_matches_target(
    puzzle: &Puzzle,
    start_colors: &[Color],
    moves: &[MoveIndex],
    target_mode: TargetMode,
) -> bool {
    let mut colors = start_colors.to_vec();
    puzzle.apply_moves(&mut colors, moves);
    is_target_solved(&colors, puzzle, target_mode)
}

fn reverse_table_seeds(puzzle: &Puzzle, target_mode: TargetMode) -> Vec<Vec<Color>> {
    match target_mode {
        TargetMode::AndroidMultiGoal => generate_android_solved_states(puzzle),
        TargetMode::PairRegion => generate_pair_region_solved_states(puzzle),
        TargetMode::Uniform | TargetMode::Android | TargetMode::AndroidPortfolio => {
            vec![puzzle.solved_colors.clone()]
        }
    }
}

fn build_phase_specs(
    puzzle: &Puzzle,
    kinds: &[PhaseKind],
    near_misses_per_face: usize,
) -> Vec<PhaseSpec> {
    let mut specs = Vec::new();
    for &kind in kinds {
        match kind {
            PhaseKind::SingleTape => {
                for tape in puzzle_tape_coords(puzzle) {
                    specs.push(make_tape_phase_spec(kind, vec![tape], near_misses_per_face));
                }
            }
            PhaseKind::CrossAxisTapePair => {
                let tapes = puzzle_tape_coords(puzzle);
                for first_index in 0..tapes.len() {
                    for second_index in first_index + 1..tapes.len() {
                        let first = tapes[first_index];
                        let second = tapes[second_index];
                        if first.axis == second.axis {
                            continue;
                        }
                        specs.push(make_tape_phase_spec(
                            kind,
                            vec![first, second],
                            near_misses_per_face,
                        ));
                    }
                }
            }
            PhaseKind::ThreeAxisTapeTriplet => {
                let tapes = puzzle_tape_coords(puzzle);
                let x_tapes = tapes.iter().copied().filter(|tape| tape.axis == 0);
                for x_tape in x_tapes {
                    for y_tape in tapes.iter().copied().filter(|tape| tape.axis == 1) {
                        for z_tape in tapes.iter().copied().filter(|tape| tape.axis == 2) {
                            specs.push(make_tape_phase_spec(
                                kind,
                                vec![x_tape, y_tape, z_tape],
                                near_misses_per_face,
                            ));
                        }
                    }
                }
            }
            PhaseKind::PairTapeSegments => {
                for tape in puzzle_tape_coords(puzzle) {
                    specs.push(make_tape_phase_spec(kind, vec![tape], near_misses_per_face));
                }
            }
            PhaseKind::AxisPairTapeSegments => {
                let tapes = puzzle_tape_coords(puzzle);
                for axis in 0..3 {
                    let axis_tapes = tapes
                        .iter()
                        .copied()
                        .filter(|tape| tape.axis == axis)
                        .collect::<Vec<_>>();
                    specs.push(make_tape_phase_spec(kind, axis_tapes, near_misses_per_face));
                }
            }
            PhaseKind::AllPairTapeSegments => {
                specs.push(make_tape_phase_spec(
                    kind,
                    puzzle_tape_coords(puzzle),
                    near_misses_per_face,
                ));
            }
            PhaseKind::ProtectedCornerArms | PhaseKind::ProtectedCornerBlock => {
                specs.extend(build_corner_phase_specs(puzzle, kind, near_misses_per_face));
            }
            PhaseKind::OneFace => {
                for face in FACES {
                    specs.push(make_phase_spec(
                        puzzle,
                        kind,
                        vec![face],
                        near_misses_per_face,
                    ));
                }
            }
            PhaseKind::OppositePair
            | PhaseKind::OppositeAndroidPair
            | PhaseKind::OppositeAndroidNearPair
            | PhaseKind::OppositeAndroidPairRegion => {
                for (left, right) in opposite_face_pairs() {
                    specs.push(make_phase_spec(
                        puzzle,
                        kind,
                        vec![left, right],
                        near_misses_per_face,
                    ));
                }
            }
            PhaseKind::AllOppositeAndroidNearPairs => {
                specs.push(make_phase_spec(
                    puzzle,
                    kind,
                    FACES.to_vec(),
                    near_misses_per_face,
                ));
            }
            PhaseKind::BinaryColorSplit => {
                for first in 1..FACES.len() {
                    for second in first + 1..FACES.len() {
                        let faces = vec![FACES[0], FACES[first], FACES[second]];
                        if binary_split_color_sets(puzzle, &faces).is_some() {
                            specs.push(make_phase_spec(puzzle, kind, faces, near_misses_per_face));
                        }
                    }
                }
            }
            PhaseKind::OppositeLayerBand | PhaseKind::OppositeLayerClassBand => {
                for (left, right) in opposite_face_pairs() {
                    if layer_band_axis(puzzle, left, right).is_some() {
                        specs.push(make_phase_spec(
                            puzzle,
                            kind,
                            vec![left, right],
                            near_misses_per_face,
                        ));
                    }
                }
            }
            PhaseKind::AdjacentPair => {
                for first_index in 0..FACES.len() {
                    for second_index in first_index + 1..FACES.len() {
                        let first = FACES[first_index];
                        let second = FACES[second_index];
                        if are_opposite_faces(first, second) {
                            continue;
                        }
                        specs.push(make_phase_spec(
                            puzzle,
                            kind,
                            vec![first, second],
                            near_misses_per_face,
                        ));
                    }
                }
            }
            PhaseKind::CornerTriplet => {
                for front_back in [Face::Front, Face::Back] {
                    for left_right in [Face::Left, Face::Right] {
                        for top_bottom in [Face::Top, Face::Bottom] {
                            specs.push(make_phase_spec(
                                puzzle,
                                kind,
                                vec![front_back, left_right, top_bottom],
                                near_misses_per_face,
                            ));
                        }
                    }
                }
            }
        }
    }
    specs
}

fn make_phase_spec(
    puzzle: &Puzzle,
    kind: PhaseKind,
    faces: Vec<Face>,
    near_misses_per_face: usize,
) -> PhaseSpec {
    let face_label = faces
        .iter()
        .map(|face| face.name())
        .collect::<Vec<_>>()
        .join("+");
    let size_label = faces
        .iter()
        .map(|&face| face_size_label(puzzle, face))
        .collect::<Vec<_>>()
        .join("+");
    let mut label = format!("{}:{face_label}:{size_label}", kind.label());
    if matches!(
        kind,
        PhaseKind::OppositeAndroidNearPair | PhaseKind::AllOppositeAndroidNearPairs
    ) {
        label.push_str(&format!(":miss<={near_misses_per_face}"));
    }
    PhaseSpec {
        kind,
        faces,
        tapes: Vec::new(),
        corner_plan: None,
        label,
        near_misses_per_face,
    }
}

fn puzzle_tape_coords(puzzle: &Puzzle) -> Vec<TapeCoord> {
    let mut tapes = puzzle
        .moves
        .iter()
        .map(|mv| TapeCoord {
            axis: mv.axis,
            layer: mv.layer,
        })
        .collect::<Vec<_>>();
    tapes.sort_unstable();
    tapes.dedup();
    tapes
}

fn tape_name(tape: TapeCoord) -> String {
    let axis = match tape.axis {
        0 => 'x',
        1 => 'y',
        2 => 'z',
        _ => '?',
    };
    format!("{axis}{}", tape.layer)
}

fn make_tape_phase_spec(
    kind: PhaseKind,
    tapes: Vec<TapeCoord>,
    near_misses_per_face: usize,
) -> PhaseSpec {
    let tape_label = tapes
        .iter()
        .copied()
        .map(tape_name)
        .collect::<Vec<_>>()
        .join("+");
    let target_label = match kind {
        PhaseKind::PairTapeSegments
        | PhaseKind::AxisPairTapeSegments
        | PhaseKind::AllPairTapeSegments => "pair-segments",
        _ => "android-segments",
    };
    PhaseSpec {
        kind,
        faces: Vec::new(),
        tapes,
        corner_plan: None,
        label: format!("{}:{tape_label}:{target_label}", kind.label()),
        near_misses_per_face,
    }
}

fn binary_split_color_sets(
    puzzle: &Puzzle,
    class_a_faces: &[Face],
) -> Option<([bool; 6], [bool; 6])> {
    let mut class_a = [false; 6];
    let mut class_b = [false; 6];
    for face in FACES {
        let color = puzzle.difficulty.face_color(face) as usize;
        if class_a_faces.contains(&face) {
            class_a[color] = true;
        } else {
            class_b[color] = true;
        }
    }

    if (0..6).any(|color| class_a[color] && class_b[color]) {
        return None;
    }
    Some((class_a, class_b))
}

fn binary_color_split_solved(puzzle: &Puzzle, colors: &[Color], spec: &PhaseSpec) -> bool {
    let Some((class_a, class_b)) = binary_split_color_sets(puzzle, &spec.faces) else {
        return false;
    };

    puzzle.stickers.iter().enumerate().all(|(index, sticker)| {
        let color = colors[index] as usize;
        let target_a = spec.faces.contains(&sticker.face);
        if class_a[color] {
            target_a
        } else if class_b[color] {
            !target_a
        } else {
            false
        }
    })
}

fn binary_color_split_score(
    puzzle: &Puzzle,
    colors: &[Color],
    spec: &PhaseSpec,
    path_len: usize,
    path_penalty: i32,
) -> i32 {
    let Some((class_a, class_b)) = binary_split_color_sets(puzzle, &spec.faces) else {
        return i32::MAX / 4;
    };

    let mut wrong = 0_i32;
    let mut solved_faces = 0_i32;
    for face in FACES {
        let target_a = spec.faces.contains(&face);
        let mut face_wrong = 0_i32;
        for &index in &puzzle.face_indexes[face.index()] {
            let color = colors[index] as usize;
            let ok = if class_a[color] {
                target_a
            } else if class_b[color] {
                !target_a
            } else {
                false
            };
            if !ok {
                face_wrong += 1;
            }
        }
        wrong += face_wrong;
        if face_wrong == 0 {
            solved_faces += 1;
        }
    }

    wrong * 1200 - solved_faces * 700 + path_len as i32 * path_penalty
}

fn layer_band_axis(
    puzzle: &Puzzle,
    first: Face,
    second: Face,
) -> Option<(u8, Face, usize, Face, usize)> {
    let dims = puzzle.layout.dims();
    let max_x = dims.rows - 1;
    let max_y = dims.cols - 1;
    let max_z = dims.layers - 1;

    if (first == Face::Front && second == Face::Back)
        || (first == Face::Back && second == Face::Front)
    {
        (max_z > 0).then_some((2, Face::Back, 0, Face::Front, max_z))
    } else if (first == Face::Left && second == Face::Right)
        || (first == Face::Right && second == Face::Left)
    {
        (max_x > 0).then_some((0, Face::Left, 0, Face::Right, max_x))
    } else if (first == Face::Top && second == Face::Bottom)
        || (first == Face::Bottom && second == Face::Top)
    {
        (max_y > 0).then_some((1, Face::Bottom, 0, Face::Top, max_y))
    } else {
        None
    }
}

fn layer_band_target_color(puzzle: &Puzzle, sticker: &Sticker, spec: &PhaseSpec) -> Option<Color> {
    if spec.faces.len() != 2 {
        return None;
    }
    let (axis, low_face, low_coord, high_face, high_coord) =
        layer_band_axis(puzzle, spec.faces[0], spec.faces[1])?;
    let low_color = puzzle.difficulty.face_color(low_face);
    let high_color = puzzle.difficulty.face_color(high_face);

    if sticker.face == low_face {
        return Some(low_color);
    }
    if sticker.face == high_face {
        return Some(high_color);
    }

    let coord = match axis {
        0 => sticker.x,
        1 => sticker.y,
        2 => sticker.z,
        _ => return None,
    };
    if coord == low_coord {
        Some(low_color)
    } else if coord == high_coord {
        Some(high_color)
    } else {
        None
    }
}

fn layer_band_solved(puzzle: &Puzzle, colors: &[Color], spec: &PhaseSpec) -> bool {
    let mut targets = 0usize;
    for (index, sticker) in puzzle.stickers.iter().enumerate() {
        if let Some(target) = layer_band_target_color(puzzle, sticker, spec) {
            targets += 1;
            if colors[index] != target {
                return false;
            }
        }
    }
    targets > 0
}

fn layer_band_class_colors(puzzle: &Puzzle, spec: &PhaseSpec) -> Option<(Color, Color)> {
    if spec.faces.len() != 2 {
        return None;
    }
    let (_, low_face, _, high_face, _) = layer_band_axis(puzzle, spec.faces[0], spec.faces[1])?;
    Some((
        puzzle.difficulty.face_color(low_face),
        puzzle.difficulty.face_color(high_face),
    ))
}

fn layer_band_class_solved(puzzle: &Puzzle, colors: &[Color], spec: &PhaseSpec) -> bool {
    let Some((low_color, high_color)) = layer_band_class_colors(puzzle, spec) else {
        return false;
    };
    let mut targets = 0usize;
    for (index, sticker) in puzzle.stickers.iter().enumerate() {
        if layer_band_target_color(puzzle, sticker, spec).is_some() {
            targets += 1;
            if colors[index] != low_color && colors[index] != high_color {
                return false;
            }
        }
    }
    targets > 0
}

fn layer_band_score(
    puzzle: &Puzzle,
    colors: &[Color],
    spec: &PhaseSpec,
    path_len: usize,
    path_penalty: i32,
) -> i32 {
    let mut wrong = 0_i32;
    let mut correct = 0_i32;
    let mut targets = 0_i32;
    for (index, sticker) in puzzle.stickers.iter().enumerate() {
        if let Some(target) = layer_band_target_color(puzzle, sticker, spec) {
            targets += 1;
            if colors[index] == target {
                correct += 1;
            } else {
                wrong += 1;
            }
        }
    }
    if targets == 0 {
        return i32::MAX / 4;
    }

    let solved_bonus = if wrong == 0 { 2500 } else { 0 };
    wrong * 1500 - correct * 20 - solved_bonus + path_len as i32 * path_penalty
}

fn layer_band_class_score(
    puzzle: &Puzzle,
    colors: &[Color],
    spec: &PhaseSpec,
    path_len: usize,
    path_penalty: i32,
) -> i32 {
    let Some((low_color, high_color)) = layer_band_class_colors(puzzle, spec) else {
        return i32::MAX / 4;
    };
    let mut wrong = 0_i32;
    let mut correct = 0_i32;
    let mut targets = 0_i32;
    for (index, sticker) in puzzle.stickers.iter().enumerate() {
        if layer_band_target_color(puzzle, sticker, spec).is_some() {
            targets += 1;
            if colors[index] == low_color || colors[index] == high_color {
                correct += 1;
            } else {
                wrong += 1;
            }
        }
    }
    if targets == 0 {
        return i32::MAX / 4;
    }

    let solved_bonus = if wrong == 0 { 2000 } else { 0 };
    wrong * 1000 - correct * 15 - solved_bonus + path_len as i32 * path_penalty
}

fn build_corner_phase_specs(
    puzzle: &Puzzle,
    kind: PhaseKind,
    near_misses_per_face: usize,
) -> Vec<PhaseSpec> {
    let dims = puzzle.layout.dims();
    let max_x = dims.rows - 1;
    let max_y = dims.cols - 1;
    let max_z = dims.layers - 1;
    let mut specs = Vec::new();

    for x in [0, max_x] {
        for y in [0, max_y] {
            for z in [0, max_z] {
                let faces = vec![
                    if x == 0 { Face::Left } else { Face::Right },
                    if y == 0 { Face::Bottom } else { Face::Top },
                    if z == 0 { Face::Back } else { Face::Front },
                ];
                let mut seed_regions = Vec::new();
                let mut arm_regions = Vec::new();
                let mut block_regions = Vec::new();
                let mut seed_indexes = Vec::new();

                for &face in &faces {
                    let seed_index = puzzle
                        .stickers
                        .iter()
                        .position(|sticker| {
                            sticker.face == face
                                && sticker.x == x
                                && sticker.y == y
                                && sticker.z == z
                        })
                        .expect("corner sticker must exist");
                    seed_indexes.push(seed_index);
                    seed_regions.push((face, vec![seed_index]));

                    let block = puzzle
                        .stickers
                        .iter()
                        .enumerate()
                        .filter_map(|(index, sticker)| {
                            if sticker.face != face {
                                return None;
                            }
                            let in_block = match face {
                                Face::Front | Face::Back => {
                                    sticker.x.abs_diff(x) <= 1 && sticker.y.abs_diff(y) <= 1
                                }
                                Face::Left | Face::Right => {
                                    sticker.y.abs_diff(y) <= 1 && sticker.z.abs_diff(z) <= 1
                                }
                                Face::Top | Face::Bottom => {
                                    sticker.x.abs_diff(x) <= 1 && sticker.z.abs_diff(z) <= 1
                                }
                            };
                            in_block.then_some(index)
                        })
                        .collect::<Vec<_>>();
                    let arms = block
                        .iter()
                        .copied()
                        .filter(|&index| {
                            let sticker = &puzzle.stickers[index];
                            sticker.x.abs_diff(x) + sticker.y.abs_diff(y) + sticker.z.abs_diff(z)
                                <= 1
                        })
                        .collect::<Vec<_>>();
                    arm_regions.push((face, arms));
                    block_regions.push((face, block));
                }

                let mut forbidden_tapes = puzzle
                    .moves
                    .iter()
                    .filter(|mv| seed_indexes.iter().any(|index| mv.cycle.contains(index)))
                    .map(|mv| TapeCoord {
                        axis: mv.axis,
                        layer: mv.layer,
                    })
                    .collect::<Vec<_>>();
                forbidden_tapes.sort_unstable();
                forbidden_tapes.dedup();

                let block_stickers = block_regions
                    .iter()
                    .map(|(_, indexes)| indexes.len())
                    .sum::<usize>();
                let arm_stickers = arm_regions
                    .iter()
                    .map(|(_, indexes)| indexes.len())
                    .sum::<usize>();
                let (stage, target_stickers) = match kind {
                    PhaseKind::ProtectedCornerArms => (CornerStage::Arms, arm_stickers),
                    PhaseKind::ProtectedCornerBlock => (CornerStage::Block, block_stickers),
                    _ => unreachable!(),
                };
                specs.push(PhaseSpec {
                    kind,
                    faces,
                    tapes: Vec::new(),
                    corner_plan: Some(CornerPlan {
                        stage,
                        seed_regions,
                        arm_regions,
                        block_regions,
                        forbidden_tapes,
                    }),
                    label: format!(
                        "{}:x{x}+y{y}+z{z}:3-to-{target_stickers}:miss<={near_misses_per_face}",
                        kind.label()
                    ),
                    near_misses_per_face,
                });
            }
        }
    }

    specs
}

fn face_size_label(puzzle: &Puzzle, face: Face) -> &'static str {
    let len = puzzle.face_indexes[face.index()].len();
    let min_len = FACES
        .iter()
        .map(|face| puzzle.face_indexes[face.index()].len())
        .min()
        .unwrap_or(len);
    let max_len = FACES
        .iter()
        .map(|face| puzzle.face_indexes[face.index()].len())
        .max()
        .unwrap_or(len);

    if min_len == max_len {
        "full"
    } else if len == max_len {
        "large"
    } else if len == min_len {
        "edge"
    } else {
        "mid"
    }
}

fn are_opposite_faces(left: Face, right: Face) -> bool {
    opposite_face_pairs()
        .into_iter()
        .any(|(a, b)| (left == a && right == b) || (left == b && right == a))
}

#[derive(Debug, Clone, Copy)]
struct FaceAnalysis {
    dominant_color: Color,
    misses: usize,
    unique_colors: usize,
}

fn analyze_face(puzzle: &Puzzle, colors: &[Color], face: Face) -> Option<FaceAnalysis> {
    let indexes = &puzzle.face_indexes[face.index()];
    let first_index = *indexes.first()?;
    let mut counts = [0usize; 6];
    for &index in indexes {
        counts[colors[index] as usize] += 1;
    }

    let mut dominant_color = colors[first_index];
    let mut dominant_count = 0usize;
    let mut unique_colors = 0usize;
    for (color, &count) in counts.iter().enumerate() {
        if count > 0 {
            unique_colors += 1;
            if count > dominant_count {
                dominant_color = color as Color;
                dominant_count = count;
            }
        }
    }

    Some(FaceAnalysis {
        dominant_color,
        misses: indexes.len() - dominant_count,
        unique_colors,
    })
}

#[derive(Debug, Clone, Copy)]
struct TapeAnalysis {
    segment_misses: usize,
    unique_penalty: usize,
    pair_misses: usize,
    segment_faces: [Face; 4],
    segment_colors: [Color; 4],
}

fn tape_cycle(puzzle: &Puzzle, tape: TapeCoord) -> Option<&[usize]> {
    puzzle
        .moves
        .iter()
        .find(|mv| mv.axis == tape.axis && mv.layer == tape.layer)
        .map(|mv| mv.cycle.as_slice())
}

fn analyze_tape(puzzle: &Puzzle, colors: &[Color], tape: TapeCoord) -> Option<TapeAnalysis> {
    let cycle = tape_cycle(puzzle, tape)?;
    if cycle.is_empty() || cycle.len() % 4 != 0 {
        return None;
    }

    let segment_len = cycle.len() / 4;
    let mut segment_colors = [0; 4];
    let mut segment_faces = [Face::Front; 4];
    let mut segment_misses = 0usize;
    let mut unique_penalty = 0usize;
    for segment in 0..4 {
        let mut counts = [0usize; 6];
        for &index in &cycle[segment * segment_len..(segment + 1) * segment_len] {
            counts[colors[index] as usize] += 1;
        }
        let (dominant_color, dominant_count) = counts
            .iter()
            .copied()
            .enumerate()
            .max_by_key(|&(color, count)| (count, std::cmp::Reverse(color)))?;
        let unique_colors = counts.iter().filter(|&&count| count > 0).count();
        segment_faces[segment] = puzzle.stickers[cycle[segment * segment_len]].face;
        segment_colors[segment] = dominant_color as Color;
        segment_misses += segment_len.saturating_sub(dominant_count);
        unique_penalty += unique_colors.saturating_sub(1);
    }

    let mut required_pairs = puzzle.difficulty.required_pairs();
    let mut pair_misses = 0usize;
    for pair in [
        sorted_pair(segment_colors[0], segment_colors[2]),
        sorted_pair(segment_colors[1], segment_colors[3]),
    ] {
        if let Some(index) = required_pairs.iter().position(|&required| required == pair) {
            required_pairs.remove(index);
        } else {
            pair_misses += 1;
        }
    }

    Some(TapeAnalysis {
        segment_misses,
        unique_penalty,
        pair_misses,
        segment_faces,
        segment_colors,
    })
}

fn partial_face_assignment_matches_android(
    puzzle: &Puzzle,
    assignment: &[Option<Color>; FACE_COUNT],
) -> bool {
    fn pair_accepts(
        assignment: &[Option<Color>; FACE_COUNT],
        faces: (Face, Face),
        pair: (Color, Color),
    ) -> bool {
        let left = assignment[faces.0.index()];
        let right = assignment[faces.1.index()];
        match (left, right) {
            (Some(left), Some(right)) => sorted_pair(left, right) == pair,
            (Some(color), None) | (None, Some(color)) => pair.0 == color || pair.1 == color,
            (None, None) => true,
        }
    }

    fn assign_pairs(
        slot: usize,
        face_pairs: &[(Face, Face)],
        required_pairs: &[(Color, Color)],
        assignment: &[Option<Color>; FACE_COUNT],
        used: &mut [bool],
    ) -> bool {
        if slot == face_pairs.len() {
            return true;
        }
        for pair_index in 0..required_pairs.len() {
            if used[pair_index]
                || !pair_accepts(assignment, face_pairs[slot], required_pairs[pair_index])
            {
                continue;
            }
            used[pair_index] = true;
            if assign_pairs(slot + 1, face_pairs, required_pairs, assignment, used) {
                return true;
            }
            used[pair_index] = false;
        }
        false
    }

    let face_pairs = opposite_face_pairs();
    let required_pairs = puzzle.difficulty.required_pairs();
    assign_pairs(
        0,
        &face_pairs,
        &required_pairs,
        assignment,
        &mut vec![false; required_pairs.len()],
    )
}

fn partial_face_assignment_pair_misses(
    puzzle: &Puzzle,
    assignment: &[Option<Color>; FACE_COUNT],
) -> usize {
    fn pair_cost(
        assignment: &[Option<Color>; FACE_COUNT],
        faces: (Face, Face),
        pair: (Color, Color),
    ) -> usize {
        match (assignment[faces.0.index()], assignment[faces.1.index()]) {
            (Some(left), Some(right)) => usize::from(sorted_pair(left, right) != pair),
            (Some(color), None) | (None, Some(color)) => {
                usize::from(pair.0 != color && pair.1 != color)
            }
            (None, None) => 0,
        }
    }

    fn search(
        slot: usize,
        face_pairs: &[(Face, Face)],
        required_pairs: &[(Color, Color)],
        assignment: &[Option<Color>; FACE_COUNT],
        used: &mut [bool],
        cost: usize,
        best: &mut usize,
    ) {
        if cost >= *best {
            return;
        }
        if slot == face_pairs.len() {
            *best = cost;
            return;
        }
        for pair_index in 0..required_pairs.len() {
            if used[pair_index] {
                continue;
            }
            used[pair_index] = true;
            search(
                slot + 1,
                face_pairs,
                required_pairs,
                assignment,
                used,
                cost + pair_cost(assignment, face_pairs[slot], required_pairs[pair_index]),
                best,
            );
            used[pair_index] = false;
        }
    }

    let face_pairs = opposite_face_pairs();
    let required_pairs = puzzle.difficulty.required_pairs();
    let mut best = usize::MAX;
    search(
        0,
        &face_pairs,
        &required_pairs,
        assignment,
        &mut vec![false; required_pairs.len()],
        0,
        &mut best,
    );
    best
}

fn tape_face_assignment(
    puzzle: &Puzzle,
    colors: &[Color],
    tapes: &[TapeCoord],
) -> Option<([Option<Color>; FACE_COUNT], usize, usize)> {
    let mut assignment = [None; FACE_COUNT];
    let mut conflicts = 0usize;
    let mut segment_misses = 0usize;
    for &tape in tapes {
        let analysis = analyze_tape(puzzle, colors, tape)?;
        segment_misses += analysis.segment_misses;
        for segment in 0..4 {
            let face_index = analysis.segment_faces[segment].index();
            let color = analysis.segment_colors[segment];
            match assignment[face_index] {
                Some(assigned) if assigned != color => conflicts += 1,
                None => assignment[face_index] = Some(color),
                _ => {}
            }
        }
    }
    Some((assignment, conflicts, segment_misses))
}

fn tapes_are_jointly_android_solved(
    puzzle: &Puzzle,
    colors: &[Color],
    tapes: &[TapeCoord],
) -> bool {
    tape_face_assignment(puzzle, colors, tapes).is_some_and(
        |(assignment, conflicts, segment_misses)| {
            segment_misses == 0
                && conflicts == 0
                && partial_face_assignment_matches_android(puzzle, &assignment)
        },
    )
}

fn tape_phase_score(
    puzzle: &Puzzle,
    colors: &[Color],
    spec: &PhaseSpec,
    path_len: usize,
    path_penalty: i32,
) -> i32 {
    let mut score = 0_i32;
    for &tape in &spec.tapes {
        let Some(analysis) = analyze_tape(puzzle, colors, tape) else {
            score += 20_000;
            continue;
        };
        score += analysis.segment_misses as i32 * 1_400;
        score += analysis.unique_penalty as i32 * 350;
        score += analysis.pair_misses as i32 * 2_400;
        if analysis.segment_misses == 0 && analysis.pair_misses == 0 {
            score -= 2_800;
        }
    }
    if let Some((assignment, conflicts, _)) = tape_face_assignment(puzzle, colors, &spec.tapes) {
        score += conflicts as i32 * 4_000;
        if conflicts == 0 && partial_face_assignment_matches_android(puzzle, &assignment) {
            score -= 2_400;
        } else {
            score += 4_000;
        }
    }
    score + path_len as i32 * path_penalty
}

#[derive(Debug, Clone, Copy)]
struct TapePairSegmentMetrics {
    pair_misses: usize,
    unique_pair_penalty: usize,
    opposite_pair_misses: usize,
}

fn tape_pair_segment_metrics(
    puzzle: &Puzzle,
    colors: &[Color],
    tape: TapeCoord,
) -> Option<TapePairSegmentMetrics> {
    let cycle = tape_cycle(puzzle, tape)?;
    if cycle.is_empty() || cycle.len() % 4 != 0 {
        return None;
    }

    let segment_len = cycle.len() / 4;
    let mut dominant_pairs = [0_u8; 4];
    let mut pair_misses = 0usize;
    let mut unique_pair_penalty = 0usize;
    for segment in 0..4 {
        let mut counts = [0usize; 3];
        for &index in &cycle[segment * segment_len..(segment + 1) * segment_len] {
            let pair_index = android_pair_index(puzzle, colors[index]) as usize;
            if let Some(count) = counts.get_mut(pair_index) {
                *count += 1;
            }
        }
        let (dominant_pair, dominant_count) = counts
            .iter()
            .copied()
            .enumerate()
            .max_by_key(|&(pair, count)| (count, std::cmp::Reverse(pair)))?;
        let unique_pairs = counts.iter().filter(|&&count| count > 0).count();
        dominant_pairs[segment] = dominant_pair as u8;
        pair_misses += segment_len.saturating_sub(dominant_count);
        unique_pair_penalty += unique_pairs.saturating_sub(1);
    }
    let opposite_pair_misses = usize::from(dominant_pairs[0] != dominant_pairs[2])
        + usize::from(dominant_pairs[1] != dominant_pairs[3]);

    Some(TapePairSegmentMetrics {
        pair_misses,
        unique_pair_penalty,
        opposite_pair_misses,
    })
}

fn tapes_are_pair_segment_solved(puzzle: &Puzzle, colors: &[Color], tapes: &[TapeCoord]) -> bool {
    tapes.iter().copied().all(|tape| {
        tape_pair_segment_metrics(puzzle, colors, tape)
            .is_some_and(|metrics| metrics.pair_misses == 0 && metrics.opposite_pair_misses == 0)
    })
}

fn tape_pair_segment_phase_score(
    puzzle: &Puzzle,
    colors: &[Color],
    spec: &PhaseSpec,
    path_len: usize,
    path_penalty: i32,
) -> i32 {
    let mut score = 0_i32;
    let mut solved_tapes = 0usize;
    for &tape in &spec.tapes {
        let Some(metrics) = tape_pair_segment_metrics(puzzle, colors, tape) else {
            score += 20_000;
            continue;
        };
        score += metrics.pair_misses as i32 * 1_100;
        score += metrics.unique_pair_penalty as i32 * 260;
        score += metrics.opposite_pair_misses as i32 * 1_800;
        if metrics.pair_misses == 0 && metrics.opposite_pair_misses == 0 {
            solved_tapes += 1;
            score -= 1_800;
        }
    }
    if solved_tapes == spec.tapes.len() && !spec.tapes.is_empty() {
        score -= 3_000;
    }
    score + path_len as i32 * path_penalty
}

fn analyze_region(colors: &[Color], indexes: &[usize]) -> Option<FaceAnalysis> {
    let first_index = *indexes.first()?;
    let mut counts = [0usize; 6];
    for &index in indexes {
        counts[colors[index] as usize] += 1;
    }
    let mut dominant_color = colors[first_index];
    let mut dominant_count = 0usize;
    let mut unique_colors = 0usize;
    for (color, &count) in counts.iter().enumerate() {
        if count > 0 {
            unique_colors += 1;
            if count > dominant_count {
                dominant_color = color as Color;
                dominant_count = count;
            }
        }
    }
    Some(FaceAnalysis {
        dominant_color,
        misses: indexes.len().saturating_sub(dominant_count),
        unique_colors,
    })
}

fn corner_regions(plan: &CornerPlan) -> &[(Face, Vec<usize>)] {
    match plan.stage {
        CornerStage::Seed => &plan.seed_regions,
        CornerStage::Arms => &plan.arm_regions,
        CornerStage::Block => &plan.block_regions,
    }
}

fn corner_face_assignment(
    colors: &[Color],
    plan: &CornerPlan,
) -> Option<([Option<Color>; FACE_COUNT], usize, usize)> {
    let mut assignment = [None; FACE_COUNT];
    let mut total_misses = 0usize;
    let mut unique_penalty = 0usize;
    for (face, indexes) in corner_regions(plan) {
        let analysis = analyze_region(colors, indexes)?;
        assignment[face.index()] = Some(analysis.dominant_color);
        total_misses += analysis.misses;
        unique_penalty += analysis.unique_colors.saturating_sub(1);
    }
    Some((assignment, total_misses, unique_penalty))
}

fn corner_phase_target_solved(
    puzzle: &Puzzle,
    colors: &[Color],
    plan: &CornerPlan,
    allowed_misses_per_region: usize,
) -> bool {
    let allowed_misses = if plan.stage == CornerStage::Seed {
        0
    } else {
        allowed_misses_per_region
    };
    let regions_ok = corner_regions(plan).iter().all(|(_, indexes)| {
        analyze_region(colors, indexes).is_some_and(|analysis| analysis.misses <= allowed_misses)
    });
    regions_ok
        && corner_face_assignment(colors, plan).is_some_and(|(assignment, _, _)| {
            partial_face_assignment_matches_android(puzzle, &assignment)
        })
}

fn corner_phase_score(
    puzzle: &Puzzle,
    colors: &[Color],
    plan: &CornerPlan,
    allowed_misses_per_region: usize,
    path_len: usize,
    path_penalty: i32,
) -> i32 {
    let Some((assignment, total_misses, unique_penalty)) = corner_face_assignment(colors, plan)
    else {
        return 50_000 + path_len as i32 * path_penalty;
    };
    let mut score = total_misses as i32 * 1_600 + unique_penalty as i32 * 450;
    score += partial_face_assignment_pair_misses(puzzle, &assignment) as i32 * 3_200;
    if partial_face_assignment_matches_android(puzzle, &assignment) {
        score -= 3_000;
    } else {
        score += 4_000;
    }
    if plan.stage != CornerStage::Seed {
        let allowed = allowed_misses_per_region.saturating_mul(corner_regions(plan).len());
        if total_misses <= allowed {
            score -= (allowed - total_misses) as i32 * 500 + 3_000;
        } else {
            score += (total_misses - allowed) as i32 * 900;
        }
    }
    score + path_len as i32 * path_penalty
}

fn opposite_regions_are_pair_class_uniform(puzzle: &Puzzle, colors: &[Color]) -> bool {
    pair_region_class_disorder(puzzle, colors) == 0
}

fn pair_region_class_disorder(puzzle: &Puzzle, colors: &[Color]) -> usize {
    let mut disorder = 0usize;
    for (left, right) in opposite_face_pairs() {
        let mut counts = [0usize; 6];
        let mut total = 0usize;
        for &index in &puzzle.face_indexes[left.index()] {
            counts[color_pair_class(puzzle.difficulty, colors[index]) as usize] += 1;
            total += 1;
        }
        for &index in &puzzle.face_indexes[right.index()] {
            counts[color_pair_class(puzzle.difficulty, colors[index]) as usize] += 1;
            total += 1;
        }
        let best_count = counts.iter().copied().max().unwrap_or(0);
        disorder += total.saturating_sub(best_count);
    }
    disorder
}

fn dominant_assignment_matches_android(
    puzzle: &Puzzle,
    dominant_assignment: &[Color; FACE_COUNT],
) -> bool {
    actual_pairs_from_uniform(dominant_assignment) == puzzle.difficulty.required_pairs()
}

fn phase_target_solved(puzzle: &Puzzle, colors: &[Color], spec: &PhaseSpec) -> bool {
    if matches!(
        spec.kind,
        PhaseKind::SingleTape | PhaseKind::CrossAxisTapePair | PhaseKind::ThreeAxisTapeTriplet
    ) {
        return tapes_are_jointly_android_solved(puzzle, colors, &spec.tapes);
    }
    if matches!(
        spec.kind,
        PhaseKind::PairTapeSegments
            | PhaseKind::AxisPairTapeSegments
            | PhaseKind::AllPairTapeSegments
    ) {
        return tapes_are_pair_segment_solved(puzzle, colors, &spec.tapes);
    }
    if matches!(
        spec.kind,
        PhaseKind::ProtectedCornerArms | PhaseKind::ProtectedCornerBlock
    ) {
        return spec.corner_plan.as_ref().is_some_and(|plan| {
            corner_phase_target_solved(puzzle, colors, plan, spec.near_misses_per_face)
        });
    }
    if matches!(spec.kind, PhaseKind::BinaryColorSplit) {
        return binary_color_split_solved(puzzle, colors, spec);
    }
    if matches!(spec.kind, PhaseKind::OppositeLayerBand) {
        return layer_band_solved(puzzle, colors, spec);
    }
    if matches!(spec.kind, PhaseKind::OppositeLayerClassBand) {
        return layer_band_class_solved(puzzle, colors, spec);
    }

    let mut face_colors = Vec::new();
    let mut dominant_colors = Vec::new();
    let mut dominant_assignment = [0; FACE_COUNT];
    for &face in &spec.faces {
        let Some(analysis) = analyze_face(puzzle, colors, face) else {
            return false;
        };
        dominant_assignment[face.index()] = analysis.dominant_color;
        dominant_colors.push(analysis.dominant_color);
        if analysis.misses == 0 {
            face_colors.push(analysis.dominant_color);
        } else if !matches!(
            spec.kind,
            PhaseKind::OppositeAndroidNearPair | PhaseKind::AllOppositeAndroidNearPairs
        ) {
            return false;
        }
    }

    match spec.kind {
        PhaseKind::SingleTape | PhaseKind::CrossAxisTapePair | PhaseKind::ThreeAxisTapeTriplet => {
            unreachable!()
        }
        PhaseKind::PairTapeSegments
        | PhaseKind::AxisPairTapeSegments
        | PhaseKind::AllPairTapeSegments => unreachable!(),
        PhaseKind::ProtectedCornerArms | PhaseKind::ProtectedCornerBlock => {
            unreachable!()
        }
        PhaseKind::BinaryColorSplit
        | PhaseKind::OppositeLayerBand
        | PhaseKind::OppositeLayerClassBand => unreachable!(),
        PhaseKind::OppositeAndroidPair => {
            face_colors.len() == 2
                && puzzle
                    .difficulty
                    .required_pairs()
                    .contains(&sorted_pair(face_colors[0], face_colors[1]))
        }
        PhaseKind::OppositeAndroidNearPair => {
            if dominant_colors.len() != 2 {
                return false;
            }
            for &face in &spec.faces {
                let Some(analysis) = analyze_face(puzzle, colors, face) else {
                    return false;
                };
                if analysis.misses > spec.near_misses_per_face {
                    return false;
                }
            }
            puzzle
                .difficulty
                .required_pairs()
                .contains(&sorted_pair(dominant_colors[0], dominant_colors[1]))
        }
        PhaseKind::OppositeAndroidPairRegion => {
            face_colors.len() == 2
                && puzzle
                    .difficulty
                    .required_pairs()
                    .contains(&sorted_pair(face_colors[0], face_colors[1]))
                && opposite_regions_are_pair_class_uniform(puzzle, colors)
        }
        PhaseKind::AllOppositeAndroidNearPairs => {
            for &face in &spec.faces {
                let Some(analysis) = analyze_face(puzzle, colors, face) else {
                    return false;
                };
                if analysis.misses > spec.near_misses_per_face {
                    return false;
                }
            }
            dominant_assignment_matches_android(puzzle, &dominant_assignment)
        }
        PhaseKind::OneFace
        | PhaseKind::OppositePair
        | PhaseKind::AdjacentPair
        | PhaseKind::CornerTriplet => true,
    }
}

fn phase_score(
    puzzle: &Puzzle,
    colors: &[Color],
    spec: &PhaseSpec,
    path_len: usize,
    path_penalty: i32,
) -> i32 {
    if matches!(
        spec.kind,
        PhaseKind::SingleTape | PhaseKind::CrossAxisTapePair | PhaseKind::ThreeAxisTapeTriplet
    ) {
        return tape_phase_score(puzzle, colors, spec, path_len, path_penalty);
    }
    if matches!(
        spec.kind,
        PhaseKind::PairTapeSegments
            | PhaseKind::AxisPairTapeSegments
            | PhaseKind::AllPairTapeSegments
    ) {
        return tape_pair_segment_phase_score(puzzle, colors, spec, path_len, path_penalty);
    }
    if matches!(
        spec.kind,
        PhaseKind::ProtectedCornerArms | PhaseKind::ProtectedCornerBlock
    ) {
        return spec
            .corner_plan
            .as_ref()
            .map(|plan| {
                corner_phase_score(
                    puzzle,
                    colors,
                    plan,
                    spec.near_misses_per_face,
                    path_len,
                    path_penalty,
                )
            })
            .unwrap_or(i32::MAX / 4);
    }
    if matches!(spec.kind, PhaseKind::BinaryColorSplit) {
        return binary_color_split_score(puzzle, colors, spec, path_len, path_penalty);
    }
    if matches!(spec.kind, PhaseKind::OppositeLayerBand) {
        return layer_band_score(puzzle, colors, spec, path_len, path_penalty);
    }
    if matches!(spec.kind, PhaseKind::OppositeLayerClassBand) {
        return layer_band_class_score(puzzle, colors, spec, path_len, path_penalty);
    }

    let mut disorder = 0_i32;
    let mut unique_penalty = 0_i32;
    let mut solved_faces = 0_i32;
    let mut face_colors = Vec::new();
    let mut dominant_colors = Vec::new();
    let mut dominant_assignment = [0; FACE_COUNT];
    let mut total_misses = 0usize;

    for &face in &spec.faces {
        let Some(analysis) = analyze_face(puzzle, colors, face) else {
            continue;
        };
        let miss = analysis.misses as i32;
        disorder += miss;
        unique_penalty += (analysis.unique_colors as i32 - 1).max(0);
        dominant_assignment[face.index()] = analysis.dominant_color;
        dominant_colors.push(analysis.dominant_color);
        total_misses += analysis.misses;
        if miss == 0 {
            solved_faces += 1;
            face_colors.push(analysis.dominant_color);
        }
    }

    let mut score = disorder * 1400 + unique_penalty * 350 - solved_faces * 1600;
    match spec.kind {
        PhaseKind::SingleTape | PhaseKind::CrossAxisTapePair | PhaseKind::ThreeAxisTapeTriplet => {
            unreachable!()
        }
        PhaseKind::PairTapeSegments
        | PhaseKind::AxisPairTapeSegments
        | PhaseKind::AllPairTapeSegments => unreachable!(),
        PhaseKind::ProtectedCornerArms | PhaseKind::ProtectedCornerBlock => unreachable!(),
        PhaseKind::BinaryColorSplit
        | PhaseKind::OppositeLayerBand
        | PhaseKind::OppositeLayerClassBand => unreachable!(),
        PhaseKind::OppositeAndroidPair => {
            if face_colors.len() == 2 {
                let pair = sorted_pair(face_colors[0], face_colors[1]);
                if puzzle.difficulty.required_pairs().contains(&pair) {
                    score -= 1200;
                } else {
                    score += 2400;
                }
            } else {
                score += 900;
            }
        }
        PhaseKind::OppositeAndroidNearPair => {
            if dominant_colors.len() == 2 {
                let pair = sorted_pair(dominant_colors[0], dominant_colors[1]);
                if puzzle.difficulty.required_pairs().contains(&pair) {
                    score -= 1000;
                } else {
                    score += 2600;
                }
                let allowed = spec.near_misses_per_face.saturating_mul(spec.faces.len());
                if total_misses <= allowed {
                    score -= (allowed - total_misses) as i32 * 180;
                } else {
                    score += (total_misses - allowed) as i32 * 900;
                }
            } else {
                score += 900;
            }
        }
        PhaseKind::OppositeAndroidPairRegion => {
            if face_colors.len() == 2 {
                let pair = sorted_pair(face_colors[0], face_colors[1]);
                if puzzle.difficulty.required_pairs().contains(&pair) {
                    score -= 1800;
                } else {
                    score += 2800;
                }
            } else {
                score += 1400;
            }
            let region_disorder = pair_region_class_disorder(puzzle, colors);
            score += region_disorder as i32 * 900;
            if region_disorder == 0 {
                score -= 2200;
            }
        }
        PhaseKind::AllOppositeAndroidNearPairs => {
            if dominant_assignment_matches_android(puzzle, &dominant_assignment) {
                score -= 3600;
            } else {
                score += 3600;
            }
            let allowed = spec.near_misses_per_face.saturating_mul(spec.faces.len());
            if total_misses <= allowed {
                score -= (allowed - total_misses) as i32 * 220;
            } else {
                score += (total_misses - allowed) as i32 * 1000;
            }
        }
        PhaseKind::OneFace
        | PhaseKind::OppositePair
        | PhaseKind::AdjacentPair
        | PhaseKind::CornerTriplet => {}
    }
    score + path_len as i32 * path_penalty
}

fn build_operation_sets(puzzle: &Puzzle, config: &SolverConfig) -> Vec<OperationSetArtifacts> {
    operation_portfolio_profiles(puzzle, config)
        .into_iter()
        .map(|profile| OperationSetArtifacts {
            profile,
            operations: build_operations(puzzle, profile),
            time_limit_ms: operation_set_time_limit_for(puzzle, config, profile),
        })
        .collect()
}

fn build_phase_operation_sets(
    puzzle: &Puzzle,
    options: &BenchOptions,
) -> Vec<OperationSetArtifacts> {
    let base_profile = options.solver.operation_profile.for_layout(puzzle.layout);
    let profiles = if options.phase_profile_portfolio {
        let candidates = if puzzle.layout == LayoutId::E && puzzle.difficulty == Difficulty::Classic
        {
            vec![
                OperationProfile::ExpandedParallel,
                OperationProfile::ExpandedWide,
            ]
        } else {
            vec![
                OperationProfile::Raw,
                OperationProfile::Basic,
                OperationProfile::Pairs,
                base_profile,
            ]
        };
        let mut profiles = Vec::new();
        for profile in candidates {
            if !profiles.contains(&profile) {
                profiles.push(profile);
            }
        }
        profiles
    } else {
        vec![base_profile]
    };

    profiles
        .into_iter()
        .map(|profile| OperationSetArtifacts {
            profile,
            operations: build_operations(puzzle, profile),
            time_limit_ms: options.phase_time_limit_ms,
        })
        .collect()
}

fn operation_portfolio_profiles(puzzle: &Puzzle, config: &SolverConfig) -> Vec<OperationProfile> {
    if config.operation_portfolio_enabled
        && config.operation_profile == OperationProfile::Auto
        && puzzle.layout == LayoutId::E
        && puzzle.difficulty == Difficulty::Classic
    {
        return vec![
            OperationProfile::ExpandedParallel,
            OperationProfile::ExpandedWide,
        ];
    }

    if config.operation_portfolio_enabled
        && config.operation_profile == OperationProfile::Auto
        && puzzle.layout == LayoutId::F
        && puzzle.difficulty == Difficulty::Classic
    {
        return vec![
            OperationProfile::Basic,
            OperationProfile::Pairs,
            OperationProfile::Conjugates,
        ];
    }

    vec![config.operation_profile.for_layout(puzzle.layout)]
}

fn operation_set_time_limit(config: &SolverConfig) -> u64 {
    if config.operation_portfolio_time_limit_ms == 0 {
        config.time_limit_ms
    } else {
        config.operation_portfolio_time_limit_ms
    }
}

fn operation_set_time_limit_for(
    puzzle: &Puzzle,
    config: &SolverConfig,
    profile: OperationProfile,
) -> u64 {
    if config.operation_portfolio_enabled
        && config.operation_profile == OperationProfile::Auto
        && puzzle.layout == LayoutId::F
        && puzzle.difficulty == Difficulty::Classic
        && profile == OperationProfile::Pairs
    {
        config.rescue_time_limit_ms
    } else {
        operation_set_time_limit(config)
    }
}

fn local_optimization_for_profile(
    puzzle: &Puzzle,
    config: &SolverConfig,
    profile: OperationProfile,
) -> (usize, usize) {
    if config.operation_portfolio_enabled
        && config.operation_profile == OperationProfile::Auto
        && puzzle.layout == LayoutId::F
        && puzzle.difficulty == Difficulty::Classic
        && profile == OperationProfile::Pairs
    {
        (F_RESCUE_LOCAL_WINDOW, F_RESCUE_LOCAL_DEPTH)
    } else {
        (config.local_window, config.local_depth)
    }
}

fn should_try_operation_portfolio_profile(
    puzzle: &Puzzle,
    config: &SolverConfig,
    best: Option<&SolveResult>,
) -> bool {
    let threshold = operation_portfolio_threshold_for(puzzle, config);
    config.operation_portfolio_enabled
        && best.is_none_or(|result| result.optimized_moves.len() >= threshold)
}

fn operation_portfolio_threshold_for(puzzle: &Puzzle, config: &SolverConfig) -> usize {
    if config.operation_profile == OperationProfile::Auto
        && config.operation_portfolio_threshold == DEFAULT_OPERATION_PORTFOLIO_THRESHOLD
        && puzzle.layout == LayoutId::F
        && puzzle.difficulty == Difficulty::Classic
    {
        DEFAULT_RESCUE_THRESHOLD
    } else {
        config.operation_portfolio_threshold
    }
}

fn should_try_pattern_db_profile(config: &SolverConfig, best: Option<&SolveResult>) -> bool {
    config.pattern_db_enabled
        && config.pattern_db_depth > 0
        && config.pattern_db_weight != 0
        && best.is_some_and(|result| result.optimized_moves.len() >= config.pattern_db_threshold)
}

fn should_try_f_pattern_db_portfolio(puzzle: &Puzzle, config: &SolverConfig) -> bool {
    config.f_pattern_db_portfolio_enabled
        && config.pattern_db_enabled
        && config.pattern_db_depth > 0
        && config.pattern_db_weight != 0
        && !config.beam_rank.needs_pattern_db()
        && config.operation_portfolio_enabled
        && config.operation_profile == OperationProfile::Auto
        && puzzle.layout == LayoutId::F
        && puzzle.difficulty == Difficulty::Classic
}

fn should_try_landmark_rescue(
    puzzle: &Puzzle,
    config: &SolverConfig,
    best: Option<&SolveResult>,
) -> bool {
    config.landmark_rescue_enabled
        && config.pattern_db_enabled
        && config.pattern_db_depth > 0
        && config.landmark_depth > 0
        && config.landmark_width > 0
        && config.landmark_candidates > 0
        && best.is_none()
        && puzzle.layout == LayoutId::E
        && puzzle.difficulty == Difficulty::Classic
}

fn hard_rescue_time_limit_for(
    puzzle: &Puzzle,
    config: &SolverConfig,
    best: Option<&SolveResult>,
) -> Option<u64> {
    if !config.hard_rescue_enabled
        || config.hard_rescue_time_limit_ms == 0
        || config.hard_rescue_tier.max_depth == 0
        || config.hard_rescue_tier.width == 0
        || config.hard_rescue_tier.restarts == 0
        || puzzle.layout != LayoutId::E
        || puzzle.difficulty != Difficulty::Classic
    {
        return None;
    }

    match best {
        None => Some(config.hard_rescue_time_limit_ms),
        Some(result) if result.optimized_moves.len() >= E_CLASSIC_QUALITY_HARD_RESCUE_THRESHOLD => {
            Some(
                config
                    .hard_rescue_time_limit_ms
                    .min(E_CLASSIC_QUALITY_HARD_RESCUE_TIME_LIMIT_MS),
            )
        }
        Some(_) => None,
    }
}

fn hard_rescue_tier_for(
    puzzle: &Puzzle,
    config: &SolverConfig,
    best: Option<&SolveResult>,
) -> Tier {
    if best.is_none() && puzzle.layout == LayoutId::E && puzzle.difficulty == Difficulty::Classic {
        return Tier {
            max_depth: config
                .hard_rescue_tier
                .max_depth
                .max(E_CLASSIC_NO_RESULT_HARD_RESCUE_DEPTH),
            width: config
                .hard_rescue_tier
                .width
                .max(E_CLASSIC_NO_RESULT_HARD_RESCUE_WIDTH),
            restarts: config
                .hard_rescue_tier
                .restarts
                .max(E_CLASSIC_NO_RESULT_HARD_RESCUE_RESTARTS),
        };
    }

    config.hard_rescue_tier
}

fn build_rescue_operations(puzzle: &Puzzle, config: &SolverConfig) -> Option<Vec<Operation>> {
    if !config.rescue_enabled || puzzle.layout != LayoutId::F {
        return None;
    }

    if config.operation_portfolio_enabled
        && config.operation_profile == OperationProfile::Auto
        && puzzle.difficulty == Difficulty::Classic
    {
        return None;
    }

    if config.operation_profile.for_layout(puzzle.layout) != OperationProfile::Basic {
        return None;
    }

    Some(build_operations(puzzle, OperationProfile::Pairs))
}

fn should_try_rescue(
    puzzle: &Puzzle,
    config: &SolverConfig,
    primary_solution_len: Option<usize>,
    best: Option<&SolveResult>,
) -> bool {
    if !config.rescue_enabled
        || puzzle.layout != LayoutId::F
        || config.operation_profile.for_layout(puzzle.layout) != OperationProfile::Basic
    {
        return false;
    }

    let solution_len = if config.operation_portfolio_enabled
        && config.operation_profile == OperationProfile::Auto
        && puzzle.difficulty == Difficulty::Classic
    {
        primary_solution_len.unwrap_or(usize::MAX)
    } else {
        best.map_or(usize::MAX, |result| result.optimized_moves.len())
    };

    solution_len >= config.rescue_threshold
}

fn build_operations(puzzle: &Puzzle, profile: OperationProfile) -> Vec<Operation> {
    let profile = profile.for_layout(puzzle.layout);
    let mut operations = Vec::new();
    let mut seen = HashSet::new();
    for index in 0..puzzle.moves.len() {
        let moves = vec![index];
        seen.insert(operation_permutation_key(puzzle, &moves));
        operations.push(Operation {
            moves: vec![index],
            path: vec![index],
            is_raw: true,
            last_move: index,
        });
    }

    if matches!(
        profile,
        OperationProfile::Pairs
            | OperationProfile::Expanded
            | OperationProfile::ExpandedParallel
            | OperationProfile::ExpandedWide
    ) {
        add_pair_operations(puzzle, &mut operations, &mut seen);
    }

    if matches!(
        profile,
        OperationProfile::Conjugates
            | OperationProfile::Expanded
            | OperationProfile::ExpandedParallel
            | OperationProfile::ExpandedWide
    ) {
        add_conjugate_operations(puzzle, &mut operations, &mut seen);
    }

    if matches!(
        profile,
        OperationProfile::ExpandedParallel | OperationProfile::ExpandedWide
    ) {
        add_parallel_pair_operations(puzzle, &mut operations, &mut seen);
    }

    if matches!(profile, OperationProfile::ExpandedWide) {
        add_parallel_triple_operations(puzzle, &mut operations, &mut seen);
    }

    if !matches!(profile, OperationProfile::Raw) {
        add_commutator_operations(puzzle, &mut operations, &mut seen);
    }

    operations
}

fn add_parallel_triple_operations(
    puzzle: &Puzzle,
    operations: &mut Vec<Operation>,
    seen: &mut HashSet<Vec<u8>>,
) {
    for axis in 0..3_u8 {
        let mut by_layer: BTreeMap<usize, Vec<MoveIndex>> = BTreeMap::new();
        for (move_index, mv) in puzzle.moves.iter().enumerate() {
            if mv.axis == axis {
                by_layer.entry(mv.layer).or_default().push(move_index);
            }
        }

        let layers = by_layer.keys().copied().collect::<Vec<_>>();
        if layers.len() < 3 {
            continue;
        }

        for a_pos in 0..layers.len() - 2 {
            for b_pos in a_pos + 1..layers.len() - 1 {
                for c_pos in b_pos + 1..layers.len() {
                    let layer_a = layers[a_pos];
                    let layer_b = layers[b_pos];
                    let layer_c = layers[c_pos];
                    for &first in &by_layer[&layer_a] {
                        for &second in &by_layer[&layer_b] {
                            for &third in &by_layer[&layer_c] {
                                add_macro_operation(
                                    puzzle,
                                    operations,
                                    seen,
                                    vec![first, second, third],
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

fn add_parallel_pair_operations(
    puzzle: &Puzzle,
    operations: &mut Vec<Operation>,
    seen: &mut HashSet<Vec<u8>>,
) {
    for first_index in 0..puzzle.moves.len() {
        for second_index in 0..puzzle.moves.len() {
            let first = &puzzle.moves[first_index];
            let second = &puzzle.moves[second_index];
            if first.axis != second.axis || first.layer == second.layer {
                continue;
            }

            add_macro_operation(puzzle, operations, seen, vec![first_index, second_index]);
        }
    }
}

fn add_pair_operations(
    puzzle: &Puzzle,
    operations: &mut Vec<Operation>,
    seen: &mut HashSet<Vec<u8>>,
) {
    for first_index in 0..puzzle.moves.len() {
        for second_index in 0..puzzle.moves.len() {
            let first = &puzzle.moves[first_index];
            let second = &puzzle.moves[second_index];
            if first.axis == second.axis || first.inverse == second_index {
                continue;
            }

            add_macro_operation(puzzle, operations, seen, vec![first_index, second_index]);
        }
    }
}

fn add_conjugate_operations(
    puzzle: &Puzzle,
    operations: &mut Vec<Operation>,
    seen: &mut HashSet<Vec<u8>>,
) {
    for first_index in 0..puzzle.moves.len() {
        for second_index in 0..puzzle.moves.len() {
            let first = &puzzle.moves[first_index];
            let second = &puzzle.moves[second_index];
            if first.axis == second.axis {
                continue;
            }

            add_macro_operation(
                puzzle,
                operations,
                seen,
                vec![first_index, second_index, first.inverse],
            );
        }
    }
}

fn add_commutator_operations(
    puzzle: &Puzzle,
    operations: &mut Vec<Operation>,
    seen: &mut HashSet<Vec<u8>>,
) {
    for first_index in 0..puzzle.moves.len() {
        for second_index in 0..puzzle.moves.len() {
            let first = &puzzle.moves[first_index];
            let second = &puzzle.moves[second_index];
            if first.axis == second.axis {
                continue;
            }

            let first_inverse = first.inverse;
            let second_inverse = second.inverse;
            let moves = vec![first_index, second_index, first_inverse, second_inverse];
            add_macro_operation(puzzle, operations, seen, moves);
        }
    }
}

fn add_macro_operation(
    puzzle: &Puzzle,
    operations: &mut Vec<Operation>,
    seen: &mut HashSet<Vec<u8>>,
    moves: Vec<MoveIndex>,
) {
    if moves.is_empty() {
        return;
    }
    if has_immediate_inverse_sequence(puzzle, &moves) {
        return;
    }

    let key = operation_permutation_key(puzzle, &moves);
    if !seen.insert(key) {
        return;
    }

    let last_move = *moves.last().expect("non-empty macro operation");
    operations.push(Operation {
        moves: moves.clone(),
        path: moves,
        is_raw: false,
        last_move,
    });
}

fn invert_operations(puzzle: &Puzzle, operations: &[Operation]) -> Vec<Operation> {
    let mut inverted = Vec::with_capacity(operations.len());
    let mut seen = HashSet::new();

    for operation in operations {
        let moves = operation
            .moves
            .iter()
            .rev()
            .map(|&move_index| puzzle.inverse_index(move_index))
            .collect::<Vec<_>>();
        if moves.is_empty() {
            continue;
        }

        let key = operation_permutation_key(puzzle, &moves);
        if !seen.insert(key) {
            continue;
        }

        let last_move = *moves.last().expect("non-empty inverted operation");
        inverted.push(Operation {
            path: moves.clone(),
            moves,
            is_raw: operation.is_raw,
            last_move,
        });
    }

    inverted
}

fn has_immediate_inverse_sequence(puzzle: &Puzzle, moves: &[MoveIndex]) -> bool {
    moves
        .windows(2)
        .any(|window| puzzle.inverse_index(window[0]) == window[1])
}

fn operation_permutation_key(puzzle: &Puzzle, moves: &[MoveIndex]) -> Vec<u8> {
    let mut permutation: Vec<u8> = (0..puzzle.stickers.len())
        .map(|index| index as u8)
        .collect();
    for &move_index in moves {
        let mv = &puzzle.moves[move_index];
        let previous = permutation.clone();
        let len = mv.cycle.len();
        for index in 0..len {
            let from = mv.cycle[index];
            let to_position = if mv.direction > 0 {
                (index + 1) % len
            } else {
                (index + len - 1) % len
            };
            let to = mv.cycle[to_position];
            permutation[to] = previous[from];
        }
    }
    permutation
}

fn scan_commutators(puzzle: &Puzzle, options: &BenchOptions) -> CommutatorScanRecord {
    let started = Instant::now();
    let mut limits = Limits::new(options.solver.max_nodes, options.solver.time_limit_ms);
    let sequences = generate_unit_sequences(puzzle, options.commutator_max_len);
    let mut seen = HashSet::new();
    let mut histogram = BTreeMap::new();
    let mut top = Vec::new();
    let mut catalog = Vec::new();
    let mut pairs_examined = 0u64;
    let mut identity_count = 0usize;
    let mut clean_3_cycles = 0usize;
    let mut double_transpositions = 0usize;
    let mut min_support = usize::MAX;
    let mut truncated = false;
    let mut reason = "complete".to_string();

    'outer: for a in &sequences {
        for b in &sequences {
            pairs_examined += 1;
            if limits.exceeded(pairs_examined) {
                truncated = true;
                reason = limits
                    .stop_reason
                    .clone()
                    .unwrap_or_else(|| "stopped".to_string());
                break 'outer;
            }

            let moves = commutator_moves(puzzle, a, b);
            let permutation = operation_permutation_key(puzzle, &moves);
            if !seen.insert(permutation.clone()) {
                continue;
            }

            let cycle_lengths = permutation_cycle_lengths_u8(&permutation);
            let support = cycle_lengths.iter().sum::<usize>();
            let cycle_type = commutator_cycle_type(&cycle_lengths);
            *histogram.entry(cycle_type.clone()).or_insert(0) += 1;

            if cycle_lengths.is_empty() {
                identity_count += 1;
                continue;
            }

            min_support = min_support.min(support);
            if cycle_lengths == [3] {
                clean_3_cycles += 1;
            }
            if cycle_lengths == [2, 2] {
                double_transpositions += 1;
            }

            let candidate = CommutatorCandidate {
                support,
                cycle_type,
                cycle_lengths,
                a: a.clone(),
                b: b.clone(),
                moves,
            };
            if matches!(candidate.cycle_lengths.as_slice(), [3] | [2, 2]) {
                catalog.push(candidate.clone());
            }
            top.push(candidate);
            if top.len() > options.commutator_top * 100 {
                top.sort_by(commutator_candidate_cmp);
                top.truncate(options.commutator_top);
            }
        }
    }

    top.sort_by(commutator_candidate_cmp);
    top.truncate(options.commutator_top);
    catalog.sort_by(commutator_candidate_cmp);

    CommutatorScanRecord {
        layout: puzzle.layout,
        difficulty: puzzle.difficulty,
        max_len: options.commutator_max_len,
        sequences: sequences.len(),
        pairs_examined,
        unique_permutations: seen.len(),
        identity_count,
        min_support: if min_support == usize::MAX {
            0
        } else {
            min_support
        },
        clean_3_cycles,
        double_transpositions,
        histogram,
        top,
        catalog,
        elapsed_ms: started.elapsed().as_millis(),
        truncated,
        reason,
    }
}

fn commutator_primitives_from_scan(
    puzzle: &Puzzle,
    record: &CommutatorScanRecord,
    include_double_transpositions: bool,
) -> Vec<CommutatorPrimitive> {
    record
        .catalog
        .iter()
        .filter(|candidate| {
            candidate.cycle_lengths == [3]
                || (include_double_transpositions && candidate.cycle_lengths == [2, 2])
        })
        .map(|candidate| {
            let permutation = operation_permutation_key(puzzle, &candidate.moves);
            let mut positions = permutation
                .iter()
                .enumerate()
                .filter_map(|(index, &source)| (source as usize != index).then_some(index))
                .collect::<Vec<_>>();
            positions.sort_unstable();
            CommutatorPrimitive {
                moves: candidate.moves.clone(),
                inverse_moves: inverse_moves(puzzle, &candidate.moves),
                positions,
            }
        })
        .collect()
}

fn inverse_moves(puzzle: &Puzzle, moves: &[MoveIndex]) -> Vec<MoveIndex> {
    moves
        .iter()
        .rev()
        .map(|&move_index| puzzle.inverse_index(move_index))
        .collect()
}

fn distinct_commutator_triples(primitives: &[CommutatorPrimitive]) -> usize {
    primitives
        .iter()
        .map(|primitive| primitive.positions.clone())
        .collect::<HashSet<_>>()
        .len()
}

fn covered_commutator_positions(primitives: &[CommutatorPrimitive]) -> usize {
    primitives
        .iter()
        .flat_map(|primitive| primitive.positions.iter().copied())
        .collect::<HashSet<_>>()
        .len()
}

fn best_commutator_target(
    puzzle: &Puzzle,
    colors: &[Color],
    target_mode: TargetMode,
) -> Vec<Color> {
    commutator_targets(puzzle, target_mode)
        .into_iter()
        .min_by_key(|target| color_mismatches(colors, target))
        .unwrap_or_else(|| puzzle.solved_colors.clone())
}

fn commutator_targets(puzzle: &Puzzle, target_mode: TargetMode) -> Vec<Vec<Color>> {
    let mut targets = Vec::new();
    for target_variant in target_variants(target_mode) {
        targets.extend(reverse_table_seeds(puzzle, target_variant));
    }
    targets.sort();
    targets.dedup();
    if targets.is_empty() {
        targets.push(puzzle.solved_colors.clone());
    }
    targets
}

fn commutator_suffix_solver_config(base: &SolverConfig, time_limit_ms: u64) -> SolverConfig {
    let mut config = base.clone();
    config.target_mode = match base.target_mode {
        TargetMode::Android | TargetMode::AndroidMultiGoal | TargetMode::AndroidPortfolio => {
            TargetMode::AndroidMultiGoal
        }
        other => other,
    };
    config.operation_profile = OperationProfile::ExpandedParallel;
    config.operation_portfolio_enabled = false;
    config.beam_rank = BeamRankMode::RingPortfolio;
    config.pattern_db_enabled = false;
    config.f_pattern_db_portfolio_enabled = false;
    config.landmark_rescue_enabled = false;
    config.hard_rescue_enabled = false;
    config.pair_region_rescue_enabled = false;
    config.rescue_enabled = false;
    config.time_limit_ms = time_limit_ms.max(1);
    config
}

fn unit_closure_solver_config(base: &SolverConfig, time_limit_ms: u64) -> SolverConfig {
    let mut config = base.clone();
    config.target_mode = match base.target_mode {
        TargetMode::Android | TargetMode::AndroidMultiGoal | TargetMode::AndroidPortfolio => {
            TargetMode::AndroidMultiGoal
        }
        other => other,
    };
    config.operation_profile = OperationProfile::Raw;
    config.operation_portfolio_enabled = false;
    config.operation_portfolio_time_limit_ms = time_limit_ms.max(1);
    config.beam_rank = BeamRankMode::RingPortfolio;
    config.pattern_db_enabled = false;
    config.f_pattern_db_portfolio_enabled = false;
    config.landmark_rescue_enabled = false;
    config.hard_rescue_enabled = false;
    config.pair_region_rescue_enabled = false;
    config.rescue_enabled = false;
    config.time_limit_ms = time_limit_ms.max(1);
    config
}

fn best_mismatches_to_targets(colors: &[Color], targets: &[Vec<Color>]) -> usize {
    targets
        .iter()
        .map(|target| color_mismatches(colors, target))
        .min()
        .unwrap_or(colors.len())
}

fn nearest_commutator_target<'a>(colors: &[Color], targets: &'a [Vec<Color>]) -> &'a [Color] {
    targets
        .iter()
        .min_by_key(|target| color_mismatches(colors, target))
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

fn color_mismatches(colors: &[Color], target: &[Color]) -> usize {
    colors
        .iter()
        .zip(target.iter())
        .filter(|(left, right)| left != right)
        .count()
}

fn audit_commutator_applicability(
    puzzle: &Puzzle,
    colors: &[Color],
    target: &[Color],
    primitives: &[CommutatorPrimitive],
    layout: LayoutId,
    difficulty: Difficulty,
    target_mode: TargetMode,
    scramble_len: usize,
    iteration: usize,
    seed: u64,
    distinct_triples: usize,
    covered_positions: usize,
) -> CommutatorApplicabilityRecord {
    let initial_mismatches = color_mismatches(colors, target);
    let mut improving_commutators = 0usize;
    let mut strong_commutators = 0usize;
    let mut best_delta = isize::MIN;
    let mut best_after_mismatches = initial_mismatches;
    let mut best_direction = "none".to_string();
    let mut best_commutator = String::new();

    for primitive in primitives {
        for (direction, moves) in [
            ("forward", primitive.moves.as_slice()),
            ("inverse", primitive.inverse_moves.as_slice()),
        ] {
            let mut next_colors = colors.to_vec();
            puzzle.apply_moves(&mut next_colors, moves);
            let after_mismatches = color_mismatches(&next_colors, target);
            let delta = initial_mismatches as isize - after_mismatches as isize;
            if delta > 0 {
                improving_commutators += 1;
            }
            if delta >= 2 {
                strong_commutators += 1;
            }
            let best_key = (delta, -(after_mismatches as isize), -(moves.len() as isize));
            let current_key = (
                best_delta,
                -(best_after_mismatches as isize),
                -(best_commutator.split_whitespace().count() as isize),
            );
            if best_key > current_key {
                best_delta = delta;
                best_after_mismatches = after_mismatches;
                best_direction = direction.to_string();
                best_commutator = puzzle.moves_text(moves);
            }
        }
    }

    if best_delta == isize::MIN {
        best_delta = 0;
    }

    CommutatorApplicabilityRecord {
        layout,
        difficulty,
        target_mode,
        scramble_len,
        iteration,
        seed,
        initial_mismatches,
        clean_3_catalog_size: primitives.len(),
        distinct_triples,
        covered_positions,
        improving_commutators,
        strong_commutators,
        best_delta,
        best_after_mismatches,
        best_direction,
        best_commutator,
    }
}

fn audit_commutator_plateau(
    puzzle: &Puzzle,
    plateau_colors: &[Color],
    target: &[Color],
    dynamic_targets: &[Vec<Color>],
    primitives: &[CommutatorPrimitive],
    layout: LayoutId,
    difficulty: Difficulty,
    target_mode: TargetMode,
    scramble_len: usize,
    iteration: usize,
    seed: u64,
    greedy: &GreedyCommutatorSolve,
) -> CommutatorPlateauRecord {
    let mismatch_positions = mismatch_positions(plateau_colors, target);
    let mismatch_set = mismatch_positions.iter().copied().collect::<HashSet<_>>();
    let plateau_mismatches = mismatch_positions.len();
    let best_dynamic_mismatches = best_mismatches_to_targets(plateau_colors, dynamic_targets);
    let residual_canonical_parity = parity_label(canonical_color_permutation_parity(
        plateau_colors,
        target,
        Some(&mismatch_positions),
    ));
    let residual_flex_parity =
        color_flexible_parity_label(plateau_colors, target, &mismatch_positions);
    let full_canonical_parity = parity_label(canonical_color_permutation_parity(
        plateau_colors,
        target,
        None,
    ));

    let mut support_touch = [0usize; 5];
    let mut support_subset_mismatch = 0usize;
    let mut support_contains_all_mismatch = 0usize;
    let mut direct_improving = 0usize;
    let mut direct_nonworsening = 0usize;
    let mut direct_best_delta = isize::MIN;
    let mut direct_best_after = plateau_mismatches;

    for primitive in primitives {
        let touch = primitive
            .positions
            .iter()
            .filter(|position| mismatch_set.contains(position))
            .count();
        if touch < support_touch.len() {
            support_touch[touch] += 1;
        }
        if !primitive.positions.is_empty()
            && primitive
                .positions
                .iter()
                .all(|position| mismatch_set.contains(position))
        {
            support_subset_mismatch += 1;
        }
        if plateau_mismatches > 0
            && mismatch_positions
                .iter()
                .all(|position| primitive.positions.contains(position))
        {
            support_contains_all_mismatch += 1;
        }

        for moves in [
            primitive.moves.as_slice(),
            primitive.inverse_moves.as_slice(),
        ] {
            let mut next_colors = plateau_colors.to_vec();
            puzzle.apply_moves(&mut next_colors, moves);
            let after = color_mismatches(&next_colors, target);
            let delta = plateau_mismatches as isize - after as isize;
            if delta > 0 {
                direct_improving += 1;
            }
            if delta >= 0 {
                direct_nonworsening += 1;
            }
            let best_key = (delta, -(after as isize), -(moves.len() as isize));
            let current_key = (direct_best_delta, -(direct_best_after as isize), 0isize);
            if best_key > current_key {
                direct_best_delta = delta;
                direct_best_after = after;
            }
        }
    }

    if direct_best_delta == isize::MIN {
        direct_best_delta = 0;
    }

    CommutatorPlateauRecord {
        layout,
        difficulty,
        target_mode,
        scramble_len,
        iteration,
        seed,
        greedy_reason: greedy.reason.clone(),
        greedy_steps: greedy.steps,
        greedy_raw_len: greedy.moves.len(),
        initial_mismatches: greedy.initial_mismatches,
        plateau_mismatches,
        best_dynamic_mismatches,
        residual_canonical_parity,
        residual_flex_parity,
        full_canonical_parity,
        support_touch_1: support_touch[1],
        support_touch_2: support_touch[2],
        support_touch_3: support_touch[3],
        support_touch_4: support_touch[4],
        support_subset_mismatch,
        support_contains_all_mismatch,
        direct_improving,
        direct_nonworsening,
        direct_best_delta,
        direct_best_after,
        mismatch_positions: join_usize(&mismatch_positions),
        mismatch_details: mismatch_details_text(
            puzzle,
            plateau_colors,
            target,
            &mismatch_positions,
        ),
        transition_counts: transition_counts_text(plateau_colors, target, &mismatch_positions),
    }
}

fn make_commutator_branch_record(
    _puzzle: &Puzzle,
    colors: &[Color],
    target: &[Color],
    primitives: &[CommutatorPrimitive],
    options: &BenchOptions,
    layout: LayoutId,
    difficulty: Difficulty,
    scramble_len: usize,
    iteration: usize,
    seed: u64,
    source: String,
    gate: usize,
    step: usize,
    ariadne_remaining: usize,
) -> CommutatorBranchAuditRecord {
    let mismatch_positions = mismatch_positions(colors, target);
    let mismatch_set = mismatch_positions.iter().copied().collect::<HashSet<_>>();
    let mut touch_bins = [0usize; 5];

    for primitive in primitives {
        let touch = primitive
            .positions
            .iter()
            .filter(|position| mismatch_set.contains(position))
            .count();
        if touch == 0 {
            continue;
        }
        if touch >= 4 {
            touch_bins[4] += 1;
        } else {
            touch_bins[touch] += 1;
        }
    }

    let filtered_primitives = touch_bins[1] + touch_bins[2] + touch_bins[3] + touch_bins[4];
    CommutatorBranchAuditRecord {
        layout,
        difficulty,
        target_mode: options.solver.target_mode,
        scramble_len,
        iteration,
        seed,
        source,
        gate,
        step,
        ariadne_remaining,
        mismatches: mismatch_positions.len(),
        filtered_primitives,
        filtered_directions: filtered_primitives * 2,
        capped_primitives: filtered_primitives.min(options.commutator_top),
        touch_1: touch_bins[1],
        touch_2: touch_bins[2],
        touch_3: touch_bins[3],
        touch_4_or_more: touch_bins[4],
    }
}

fn residual_support_stats(
    mismatch_positions: &[usize],
    primitives: &[CommutatorPrimitive],
) -> (bool, usize, usize) {
    let mismatch_set = mismatch_positions.iter().copied().collect::<HashSet<_>>();
    let mut exact_support_available = false;
    let mut subset_support_count = 0usize;
    let mut contains_all_support_count = 0usize;

    for primitive in primitives {
        if !primitive.positions.is_empty()
            && primitive
                .positions
                .iter()
                .all(|position| mismatch_set.contains(position))
        {
            subset_support_count += 1;
        }
        if !mismatch_positions.is_empty()
            && mismatch_positions
                .iter()
                .all(|position| primitive.positions.contains(position))
        {
            contains_all_support_count += 1;
        }
        if primitive.positions.len() == mismatch_positions.len()
            && primitive
                .positions
                .iter()
                .all(|position| mismatch_set.contains(position))
        {
            exact_support_available = true;
        }
    }

    (
        exact_support_available,
        subset_support_count,
        contains_all_support_count,
    )
}

fn target_mismatch_sensitivity(colors: &[Color], targets: &[Vec<Color>]) -> (usize, usize, usize) {
    let mut min_mismatches = usize::MAX;
    let mut max_mismatches = 0usize;
    let mut min_count = 0usize;

    for target in targets {
        let mismatches = color_mismatches(colors, target);
        if mismatches < min_mismatches {
            min_mismatches = mismatches;
            min_count = 1;
        } else if mismatches == min_mismatches {
            min_count += 1;
        }
        max_mismatches = max_mismatches.max(mismatches);
    }

    if min_mismatches == usize::MAX {
        (colors.len(), colors.len(), 0)
    } else {
        (min_mismatches, max_mismatches, min_count)
    }
}

fn mismatch_positions(colors: &[Color], target: &[Color]) -> Vec<usize> {
    colors
        .iter()
        .zip(target.iter())
        .enumerate()
        .filter_map(|(index, (color, target))| (color != target).then_some(index))
        .collect()
}

fn canonical_color_permutation_parity(
    colors: &[Color],
    target: &[Color],
    subset: Option<&[usize]>,
) -> bool {
    let positions = subset
        .map(|subset| subset.to_vec())
        .unwrap_or_else(|| (0..colors.len()).collect::<Vec<_>>());
    if positions.len() < 2 {
        return false;
    }

    let mut local_index = HashMap::new();
    for (local, &position) in positions.iter().enumerate() {
        local_index.insert(position, local);
    }

    let mut permutation = vec![0usize; positions.len()];
    for color in TARGET_COLORS {
        let mut sources = positions
            .iter()
            .copied()
            .filter(|&position| colors[position] == color)
            .collect::<Vec<_>>();
        let mut destinations = positions
            .iter()
            .copied()
            .filter(|&position| target[position] == color)
            .collect::<Vec<_>>();
        sources.sort_unstable();
        destinations.sort_unstable();
        if sources.len() != destinations.len() {
            return false;
        }
        for (source, destination) in sources.into_iter().zip(destinations.into_iter()) {
            let source_local = local_index[&source];
            let destination_local = local_index[&destination];
            permutation[source_local] = destination_local;
        }
    }

    permutation_is_odd(&permutation)
}

fn permutation_is_odd(permutation: &[usize]) -> bool {
    let mut inversions = 0usize;
    for i in 0..permutation.len() {
        for j in i + 1..permutation.len() {
            if permutation[i] > permutation[j] {
                inversions += 1;
            }
        }
    }
    inversions % 2 == 1
}

fn parity_label(is_odd: bool) -> String {
    if is_odd {
        "odd".to_string()
    } else {
        "even".to_string()
    }
}

fn color_flexible_parity_label(colors: &[Color], target: &[Color], positions: &[usize]) -> String {
    if positions.len() < 2 {
        return "even-only".to_string();
    }
    let canonical_odd = canonical_color_permutation_parity(colors, target, Some(positions));
    for color in TARGET_COLORS {
        let current_count = positions
            .iter()
            .filter(|&&position| colors[position] == color)
            .count();
        let target_count = positions
            .iter()
            .filter(|&&position| target[position] == color)
            .count();
        if current_count != target_count {
            return "invalid-color-counts".to_string();
        }
        if current_count >= 2 {
            return "both".to_string();
        }
    }
    if canonical_odd {
        "odd-only".to_string()
    } else {
        "even-only".to_string()
    }
}

fn mismatch_details_text(
    puzzle: &Puzzle,
    colors: &[Color],
    target: &[Color],
    positions: &[usize],
) -> String {
    positions
        .iter()
        .map(|&position| {
            format!(
                "{}:{}>{}",
                sticker_ref_text(puzzle, position),
                color_label(colors[position]),
                color_label(target[position])
            )
        })
        .collect::<Vec<_>>()
        .join(";")
}

fn transition_counts_text(colors: &[Color], target: &[Color], positions: &[usize]) -> String {
    let mut counts = BTreeMap::new();
    for &position in positions {
        let key = format!(
            "{}>{}",
            color_label(colors[position]),
            color_label(target[position])
        );
        *counts.entry(key).or_insert(0) += 1;
    }
    join_counts(&counts)
}

fn color_label(color: Color) -> &'static str {
    match color {
        WHITE => "W",
        RED => "R",
        BLUE => "B",
        MAGENTA => "M",
        GREEN => "G",
        YELLOW => "Y",
        _ => "?",
    }
}

fn greedy_commutator_solve(
    puzzle: &Puzzle,
    colors: &[Color],
    targets: &[Vec<Color>],
    primitives: &[CommutatorPrimitive],
    max_steps: usize,
    plateau_lookahead: usize,
) -> GreedyCommutatorSolve {
    let initial_mismatches = best_mismatches_to_targets(colors, targets);
    let mut current_colors = colors.to_vec();
    let mut current_mismatches = initial_mismatches;
    let mut solution_moves = Vec::new();
    let mut deltas = Vec::new();
    let mut primitive_steps = 0usize;

    while primitive_steps < max_steps {
        if current_mismatches == 0 {
            return GreedyCommutatorSolve {
                initial_mismatches,
                steps: primitive_steps,
                moves: solution_moves,
                deltas,
                reason: "found".to_string(),
            };
        }

        let mut best_delta = 0isize;
        let mut best_after_mismatches = current_mismatches;
        let mut best_moves = Vec::new();
        for primitive in primitives {
            for moves in [
                primitive.moves.as_slice(),
                primitive.inverse_moves.as_slice(),
            ] {
                let mut next_colors = current_colors.clone();
                puzzle.apply_moves(&mut next_colors, moves);
                let after_mismatches = best_mismatches_to_targets(&next_colors, targets);
                let delta = current_mismatches as isize - after_mismatches as isize;
                let best_key = (delta, -(after_mismatches as isize), -(moves.len() as isize));
                let current_key = (
                    best_delta,
                    -(best_after_mismatches as isize),
                    -(best_moves.len() as isize),
                );
                if best_key > current_key {
                    best_delta = delta;
                    best_after_mismatches = after_mismatches;
                    best_moves = moves.to_vec();
                }
            }
        }

        if best_delta <= 0 || best_moves.is_empty() {
            if plateau_lookahead > 0 {
                if let Some((repair_delta, repair_after, repair_moves, repair_colors)) =
                    best_plateau_repair(
                        puzzle,
                        &current_colors,
                        current_mismatches,
                        targets,
                        primitives,
                        plateau_lookahead,
                    )
                {
                    current_colors = repair_colors;
                    current_mismatches = repair_after;
                    solution_moves.extend(repair_moves);
                    deltas.push(repair_delta);
                    primitive_steps += 2;
                    continue;
                }
            }
            return GreedyCommutatorSolve {
                initial_mismatches,
                steps: primitive_steps,
                moves: solution_moves,
                deltas,
                reason: "plateau".to_string(),
            };
        }

        puzzle.apply_moves(&mut current_colors, &best_moves);
        current_mismatches = best_after_mismatches;
        solution_moves.extend(best_moves);
        deltas.push(best_delta);
        primitive_steps += 1;
    }

    GreedyCommutatorSolve {
        initial_mismatches,
        steps: primitive_steps,
        moves: solution_moves,
        deltas,
        reason: "step_limit".to_string(),
    }
}

fn commutator_endgame_search(
    puzzle: &Puzzle,
    colors: &[Color],
    targets: &[Vec<Color>],
    primitives: &[CommutatorPrimitive],
    options: &BenchOptions,
) -> CommutatorEndgameSearchResult {
    let mut nodes = 0u64;
    let mut limits = Limits::new(
        options.solver.max_nodes,
        options.commutator_endgame_time_limit_ms.max(1),
    );
    let initial_mismatches = best_mismatches_to_targets(colors, targets);
    if initial_mismatches == 0 {
        return CommutatorEndgameSearchResult {
            found: true,
            reason: "already_solved".to_string(),
            moves: Vec::new(),
            depth: 0,
            nodes,
            best_mismatches: 0,
        };
    }
    if primitives.is_empty() {
        return CommutatorEndgameSearchResult {
            found: false,
            reason: "no_primitives".to_string(),
            moves: Vec::new(),
            depth: 0,
            nodes,
            best_mismatches: initial_mismatches,
        };
    }

    let initial_score = score_state(
        colors,
        puzzle,
        options.solver.target_mode,
        options.solver.region_pair_weight,
        None,
    );
    let mut frontier = vec![CommutatorEndgameEntry {
        colors: colors.to_vec(),
        moves: Vec::new(),
        mismatches: initial_mismatches,
        score: initial_score,
    }];
    let mut seen = HashSet::new();
    seen.insert(state_key(colors));
    let mut best_mismatches = initial_mismatches;
    let heap_capacity = options
        .commutator_endgame_width
        .saturating_mul(8)
        .max(options.commutator_endgame_width);
    let mut serial = 0usize;

    for depth in 0..options.commutator_endgame_depth {
        let mut heap = BinaryHeap::new();

        for entry in &frontier {
            let target = nearest_commutator_target(&entry.colors, targets);
            let mismatch_positions = mismatch_positions(&entry.colors, target);
            let mismatch_set = mismatch_positions.iter().copied().collect::<HashSet<_>>();

            for primitive in primitives {
                if !mismatch_set.is_empty()
                    && !primitive
                        .positions
                        .iter()
                        .any(|position| mismatch_set.contains(position))
                {
                    continue;
                }

                for moves in [
                    primitive.moves.as_slice(),
                    primitive.inverse_moves.as_slice(),
                ] {
                    nodes += 1;
                    if limits.exceeded(nodes) {
                        return CommutatorEndgameSearchResult {
                            found: false,
                            reason: limits
                                .stop_reason
                                .clone()
                                .unwrap_or_else(|| "limit".to_string()),
                            moves: Vec::new(),
                            depth,
                            nodes,
                            best_mismatches,
                        };
                    }

                    let mut next_colors = entry.colors.clone();
                    puzzle.apply_moves(&mut next_colors, moves);
                    let key = state_key(&next_colors);
                    if seen.contains(&key) {
                        continue;
                    }

                    let next_mismatches = best_mismatches_to_targets(&next_colors, targets);
                    best_mismatches = best_mismatches.min(next_mismatches);
                    let mut next_moves = entry.moves.clone();
                    next_moves.extend_from_slice(moves);

                    if next_mismatches == 0 {
                        return CommutatorEndgameSearchResult {
                            found: true,
                            reason: "found".to_string(),
                            moves: next_moves,
                            depth: depth + 1,
                            nodes,
                            best_mismatches: 0,
                        };
                    }

                    let score = score_state(
                        &next_colors,
                        puzzle,
                        options.solver.target_mode,
                        options.solver.region_pair_weight,
                        None,
                    );
                    let rank = CommutatorEndgameRank {
                        mismatches: next_mismatches,
                        score,
                        unit_len: next_moves.len(),
                    };
                    let item = CommutatorEndgameHeapItem {
                        rank,
                        serial,
                        entry: CommutatorEndgameEntry {
                            colors: next_colors,
                            moves: next_moves,
                            mismatches: next_mismatches,
                            score,
                        },
                    };
                    serial = serial.wrapping_add(1);
                    push_bounded_endgame_candidate(&mut heap, heap_capacity, item);
                }
            }
        }

        let mut candidates = heap.into_vec();
        candidates.sort_unstable_by(|left, right| {
            left.rank
                .cmp(&right.rank)
                .then_with(|| left.serial.cmp(&right.serial))
        });

        let mut next_frontier = Vec::new();
        let mut layer_keys = HashSet::new();
        for item in candidates {
            let key = state_key(&item.entry.colors);
            if !layer_keys.insert(key) {
                continue;
            }
            if !seen.insert(key) {
                continue;
            }
            next_frontier.push(item.entry);
            if next_frontier.len() >= options.commutator_endgame_width {
                break;
            }
        }

        if next_frontier.is_empty() {
            return CommutatorEndgameSearchResult {
                found: false,
                reason: "no_frontier".to_string(),
                moves: Vec::new(),
                depth: depth + 1,
                nodes,
                best_mismatches,
            };
        }

        frontier = next_frontier;
    }

    let best_entry = frontier
        .iter()
        .min_by_key(|entry| {
            (
                entry.mismatches,
                entry.score,
                entry.moves.len(),
                state_key(&entry.colors),
            )
        })
        .cloned();

    CommutatorEndgameSearchResult {
        found: false,
        reason: "depth_limit".to_string(),
        moves: best_entry.map(|entry| entry.moves).unwrap_or_default(),
        depth: options.commutator_endgame_depth,
        nodes,
        best_mismatches,
    }
}

fn commutator_setup_tail_solve(
    puzzle: &Puzzle,
    colors: &[Color],
    targets: &[Vec<Color>],
    primitives: &[CommutatorPrimitive],
    max_steps: usize,
    setup_depth: usize,
    options: &BenchOptions,
) -> CommutatorHelperTailResult {
    let mut nodes = 0u64;
    let mut limits = Limits::new(
        options.solver.max_nodes,
        options.commutator_endgame_time_limit_ms.max(1),
    );
    let mut setup_sequences = vec![Vec::new()];
    setup_sequences.extend(generate_unit_sequences(puzzle, setup_depth));

    let mut current_colors = colors.to_vec();
    let mut current_mismatches = best_mismatches_to_targets(&current_colors, targets);
    let mut solution_moves = Vec::new();

    for step in 0..max_steps {
        if current_mismatches == 0 {
            return CommutatorHelperTailResult {
                found: true,
                reason: "found".to_string(),
                moves: solution_moves,
                steps: step,
                nodes,
            };
        }

        let mut best_delta = 0isize;
        let mut best_after = current_mismatches;
        let mut best_moves = Vec::new();
        let mut best_colors = Vec::new();

        for setup in &setup_sequences {
            let mut setup_colors = current_colors.clone();
            puzzle.apply_moves(&mut setup_colors, setup);
            let setup_target = nearest_commutator_target(&setup_colors, targets);
            let setup_mismatches = mismatch_positions(&setup_colors, setup_target);
            let setup_mismatch_set = setup_mismatches.iter().copied().collect::<HashSet<_>>();
            let undo_setup = inverse_moves(puzzle, setup);

            for primitive in primitives {
                if !setup_mismatch_set.is_empty()
                    && !primitive
                        .positions
                        .iter()
                        .any(|position| setup_mismatch_set.contains(position))
                {
                    continue;
                }

                for primitive_moves in [
                    primitive.moves.as_slice(),
                    primitive.inverse_moves.as_slice(),
                ] {
                    nodes += 1;
                    if limits.exceeded(nodes) {
                        return CommutatorHelperTailResult {
                            found: false,
                            reason: limits
                                .stop_reason
                                .clone()
                                .unwrap_or_else(|| "limit".to_string()),
                            moves: solution_moves,
                            steps: step,
                            nodes,
                        };
                    }

                    let mut next_colors = setup_colors.clone();
                    puzzle.apply_moves(&mut next_colors, primitive_moves);
                    puzzle.apply_moves(&mut next_colors, &undo_setup);
                    let after = best_mismatches_to_targets(&next_colors, targets);
                    let delta = current_mismatches as isize - after as isize;
                    let operation_len = setup.len() + primitive_moves.len() + undo_setup.len();
                    let best_key = (delta, -(after as isize), -(operation_len as isize));
                    let current_key = (
                        best_delta,
                        -(best_after as isize),
                        -(best_moves.len() as isize),
                    );
                    if best_key > current_key {
                        best_delta = delta;
                        best_after = after;
                        best_moves = setup.clone();
                        best_moves.extend_from_slice(primitive_moves);
                        best_moves.extend_from_slice(&undo_setup);
                        best_colors = next_colors;
                    }
                }
            }
        }

        if best_delta <= 0 || best_moves.is_empty() {
            return CommutatorHelperTailResult {
                found: false,
                reason: "plateau".to_string(),
                moves: solution_moves,
                steps: step,
                nodes,
            };
        }

        current_colors = best_colors;
        current_mismatches = best_after;
        solution_moves.extend(best_moves);
    }

    CommutatorHelperTailResult {
        found: current_mismatches == 0,
        reason: if current_mismatches == 0 {
            "found".to_string()
        } else {
            "step_limit".to_string()
        },
        moves: solution_moves,
        steps: max_steps,
        nodes,
    }
}

fn push_bounded_endgame_candidate(
    heap: &mut BinaryHeap<CommutatorEndgameHeapItem>,
    capacity: usize,
    item: CommutatorEndgameHeapItem,
) {
    if heap.len() < capacity {
        heap.push(item);
        return;
    }

    let Some(worst) = heap.peek() else {
        heap.push(item);
        return;
    };
    if item.rank < worst.rank {
        heap.pop();
        heap.push(item);
    }
}

fn best_plateau_repair(
    puzzle: &Puzzle,
    colors: &[Color],
    current_mismatches: usize,
    targets: &[Vec<Color>],
    primitives: &[CommutatorPrimitive],
    first_limit: usize,
) -> Option<(isize, usize, Vec<MoveIndex>, Vec<Color>)> {
    let mut first_candidates = Vec::new();
    for primitive in primitives {
        for moves in [
            primitive.moves.as_slice(),
            primitive.inverse_moves.as_slice(),
        ] {
            let mut next_colors = colors.to_vec();
            puzzle.apply_moves(&mut next_colors, moves);
            let after_mismatches = best_mismatches_to_targets(&next_colors, targets);
            first_candidates.push(PlateauRepairCandidate {
                after_mismatches,
                moves: moves.to_vec(),
                colors: next_colors,
            });
        }
    }

    first_candidates.sort_unstable_by(|left, right| {
        left.after_mismatches
            .cmp(&right.after_mismatches)
            .then_with(|| left.moves.len().cmp(&right.moves.len()))
            .then_with(|| left.moves.cmp(&right.moves))
    });
    first_candidates.truncate(first_limit);

    let mut best_delta = 0isize;
    let mut best_after = current_mismatches;
    let mut best_moves = Vec::new();
    let mut best_colors = Vec::new();

    for first in &first_candidates {
        for primitive in primitives {
            for second_moves in [
                primitive.moves.as_slice(),
                primitive.inverse_moves.as_slice(),
            ] {
                let mut next_colors = first.colors.clone();
                puzzle.apply_moves(&mut next_colors, second_moves);
                let after_mismatches = best_mismatches_to_targets(&next_colors, targets);
                let delta = current_mismatches as isize - after_mismatches as isize;
                let combined_len = first.moves.len() + second_moves.len();
                let best_key = (
                    delta,
                    -(after_mismatches as isize),
                    -(combined_len as isize),
                );
                let current_key = (
                    best_delta,
                    -(best_after as isize),
                    -(best_moves.len() as isize),
                );
                if best_key > current_key {
                    best_delta = delta;
                    best_after = after_mismatches;
                    best_moves = first.moves.clone();
                    best_moves.extend_from_slice(second_moves);
                    best_colors = next_colors;
                }
            }
        }
    }

    (best_delta > 0).then_some((best_delta, best_after, best_moves, best_colors))
}

fn generate_unit_sequences(puzzle: &Puzzle, max_len: usize) -> Vec<Vec<MoveIndex>> {
    fn extend(
        puzzle: &Puzzle,
        max_len: usize,
        out: &mut Vec<Vec<MoveIndex>>,
        current: &mut Vec<MoveIndex>,
    ) {
        if !current.is_empty() {
            out.push(current.clone());
        }
        if current.len() >= max_len {
            return;
        }
        for move_index in 0..puzzle.moves.len() {
            if current
                .last()
                .is_some_and(|&previous| puzzle.inverse_index(previous) == move_index)
            {
                continue;
            }
            current.push(move_index);
            extend(puzzle, max_len, out, current);
            current.pop();
        }
    }

    let mut out = Vec::new();
    extend(puzzle, max_len, &mut out, &mut Vec::new());
    out
}

fn commutator_moves(puzzle: &Puzzle, a: &[MoveIndex], b: &[MoveIndex]) -> Vec<MoveIndex> {
    let mut moves = Vec::with_capacity((a.len() + b.len()) * 2);
    moves.extend_from_slice(a);
    moves.extend_from_slice(b);
    moves.extend(
        a.iter()
            .rev()
            .map(|&move_index| puzzle.inverse_index(move_index)),
    );
    moves.extend(
        b.iter()
            .rev()
            .map(|&move_index| puzzle.inverse_index(move_index)),
    );
    moves
}

fn permutation_cycle_lengths_u8(permutation: &[u8]) -> Vec<usize> {
    let mut visited = vec![false; permutation.len()];
    let mut lengths = Vec::new();
    for start in 0..permutation.len() {
        if visited[start] {
            continue;
        }
        let mut current = start;
        let mut len = 0usize;
        while !visited[current] {
            visited[current] = true;
            len += 1;
            current = permutation[current] as usize;
        }
        if len > 1 {
            lengths.push(len);
        }
    }
    lengths.sort_unstable();
    lengths
}

fn commutator_cycle_type(cycle_lengths: &[usize]) -> String {
    match cycle_lengths {
        [] => "identity".to_string(),
        [2] => "single-transposition".to_string(),
        [3] => "clean-3-cycle".to_string(),
        [2, 2] => "double-transposition".to_string(),
        [len] => format!("clean-{len}-cycle"),
        _ => cycle_lengths
            .iter()
            .map(usize::to_string)
            .collect::<Vec<_>>()
            .join("+"),
    }
}

fn commutator_candidate_cmp(left: &CommutatorCandidate, right: &CommutatorCandidate) -> Ordering {
    commutator_cycle_priority(&left.cycle_lengths)
        .cmp(&commutator_cycle_priority(&right.cycle_lengths))
        .then_with(|| left.support.cmp(&right.support))
        .then_with(|| left.moves.len().cmp(&right.moves.len()))
        .then_with(|| left.a.cmp(&right.a))
        .then_with(|| left.b.cmp(&right.b))
}

fn commutator_cycle_priority(cycle_lengths: &[usize]) -> usize {
    match cycle_lengths {
        [3] => 0,
        [2, 2] => 1,
        [2] => 2,
        [_] => 3,
        [] => 99,
        _ => 4,
    }
}

fn apply_move_to_permutation(puzzle: &Puzzle, permutation: &mut [u8], move_index: MoveIndex) {
    let mv = &puzzle.moves[move_index];
    let previous = permutation.to_vec();
    let len = mv.cycle.len();
    for index in 0..len {
        let from = mv.cycle[index];
        let to_position = if mv.direction > 0 {
            (index + 1) % len
        } else {
            (index + len - 1) % len
        };
        let to = mv.cycle[to_position];
        permutation[to] = previous[from];
    }
}

fn apply_macro_to_permutation(
    puzzle: &Puzzle,
    permutation: &mut [u8],
    move_index: MoveIndex,
    shift: usize,
) {
    let mv = &puzzle.moves[move_index];
    let previous = permutation.to_vec();
    let len = mv.cycle.len();
    let shift = shift % len;
    for index in 0..len {
        let from = mv.cycle[index];
        let to_position = if mv.direction > 0 {
            (index + shift) % len
        } else {
            (index + len - shift) % len
        };
        let to = mv.cycle[to_position];
        permutation[to] = previous[from];
    }
}

fn apply_macro_to_colors(
    puzzle: &Puzzle,
    colors: &mut [Color],
    move_index: MoveIndex,
    shift: usize,
) {
    let mv = &puzzle.moves[move_index];
    let previous = colors.to_vec();
    let len = mv.cycle.len();
    let shift = shift % len;
    for index in 0..len {
        let from = mv.cycle[index];
        let to_position = if mv.direction > 0 {
            (index + shift) % len
        } else {
            (index + len - shift) % len
        };
        let to = mv.cycle[to_position];
        colors[to] = previous[from];
    }
}

fn build_macro_ops(puzzle: &Puzzle, shift: usize) -> Vec<MacroOp> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for move_index in 0..puzzle.moves.len() {
        let mut permutation = (0..puzzle.stickers.len())
            .map(|index| index as u8)
            .collect::<Vec<_>>();
        apply_macro_to_permutation(puzzle, &mut permutation, move_index, shift);
        if !seen.insert(permutation.clone()) {
            continue;
        }

        out.push(MacroOp { move_index, shift });
    }
    out
}

fn audit_macro_subgroup(
    puzzle: &Puzzle,
    shift: usize,
    max_depth: usize,
    max_nodes: u64,
) -> MacroSubgroupAuditResult {
    let started = Instant::now();
    let macros = build_macro_ops(puzzle, shift);
    let identity_permutation = (0..puzzle.stickers.len())
        .map(|index| index as u8)
        .collect::<Vec<_>>();
    let mut permutation_seen = HashSet::new();
    let mut color_seen = HashSet::new();
    let mut depth_counts = vec![0usize; max_depth + 1];
    let mut color_depth_counts = vec![0usize; max_depth + 1];
    let mut queue = VecDeque::new();

    permutation_seen.insert(identity_permutation.clone());
    color_seen.insert(state_key(&puzzle.solved_colors));
    depth_counts[0] = 1;
    color_depth_counts[0] = 1;
    queue.push_back(MacroSubgroupNode {
        permutation: identity_permutation,
        colors: puzzle.solved_colors.clone(),
        depth: 0,
        last_macro: None,
    });

    let mut explored = 0u64;
    let mut truncated = false;
    let mut depth_limited = false;

    while let Some(node) = queue.pop_front() {
        if explored >= max_nodes {
            truncated = true;
            break;
        }
        explored += 1;
        if node.depth >= max_depth {
            depth_limited = true;
            continue;
        }

        for (macro_index, macro_op) in macros.iter().enumerate() {
            if node.last_macro.is_some_and(|last| {
                macros[last].move_index == puzzle.inverse_index(macro_op.move_index)
            }) {
                continue;
            }

            let mut next_permutation = node.permutation.clone();
            apply_macro_to_permutation(
                puzzle,
                &mut next_permutation,
                macro_op.move_index,
                macro_op.shift,
            );

            let mut next_colors = node.colors.clone();
            apply_macro_to_colors(
                puzzle,
                &mut next_colors,
                macro_op.move_index,
                macro_op.shift,
            );

            let next_depth = node.depth + 1;
            let is_new_permutation = permutation_seen.insert(next_permutation.clone());
            if color_seen.insert(state_key(&next_colors)) {
                color_depth_counts[next_depth] += 1;
            }

            if is_new_permutation {
                depth_counts[next_depth] += 1;
                queue.push_back(MacroSubgroupNode {
                    permutation: next_permutation,
                    colors: next_colors,
                    depth: next_depth,
                    last_macro: Some(macro_index),
                });
            }
        }
    }

    MacroSubgroupAuditResult {
        shift,
        macro_count: macros.len(),
        explored,
        permutation_states: permutation_seen.len(),
        color_states: color_seen.len(),
        depth_counts,
        color_depth_counts,
        exhausted: queue.is_empty() && !truncated && !depth_limited,
        depth_limited,
        truncated,
        elapsed_ms: started.elapsed().as_millis(),
    }
}

fn build_macro_target_artifacts(
    puzzle: &Puzzle,
    target_mode: TargetMode,
    shift: usize,
    max_depth: usize,
    pattern_depth: usize,
    config: &SolverConfig,
) -> io::Result<MacroTargetArtifacts> {
    let started = Instant::now();
    let macro_ops = build_macro_ops(puzzle, shift);
    let mut build_nodes = 0u64;
    let mut build_limits = Limits::new(config.max_nodes, config.time_limit_ms.max(120_000));
    let (table, target_colors, depth_counts) = build_macro_color_table(
        puzzle,
        target_mode,
        &macro_ops,
        max_depth,
        &mut build_nodes,
        &mut build_limits,
    )?;
    let build_ms = started.elapsed().as_millis();

    let projection_started = Instant::now();
    let mut projection_build_nodes = 0u64;
    let projection_db = if pattern_depth > 0 {
        let mut projection_limits =
            Limits::new(config.max_nodes, config.time_limit_ms.max(120_000));
        Some(build_macro_target_projection_db(
            puzzle,
            &target_colors,
            config.pattern_db_projection,
            pattern_depth,
            &mut projection_build_nodes,
            &mut projection_limits,
        )?)
    } else {
        None
    };
    let projection_build_ms = projection_started.elapsed().as_millis();

    Ok(MacroTargetArtifacts {
        table,
        target_colors,
        macro_ops,
        depth_counts,
        build_nodes,
        build_ms,
        projection_db,
        projection_build_nodes,
        projection_build_ms,
    })
}

fn build_macro_color_table(
    puzzle: &Puzzle,
    target_mode: TargetMode,
    macro_ops: &[MacroOp],
    max_depth: usize,
    nodes: &mut u64,
    limits: &mut Limits,
) -> io::Result<(HashMap<Key, PathBits>, Vec<Vec<Color>>, Vec<usize>)> {
    let mut table: HashMap<Key, PathBits> = HashMap::new();
    let mut target_colors = Vec::new();
    let mut depth_counts = vec![0usize; max_depth + 1];
    let mut frontier = Vec::new();

    for seed in reverse_table_seeds(puzzle, target_mode) {
        let key = state_key(&seed);
        if table.insert(key, PathBits::default()).is_none() {
            depth_counts[0] += 1;
            target_colors.push(seed.clone());
            frontier.push(MacroTargetEntry {
                colors: seed,
                path: PathBits::default(),
                last_macro: None,
            });
        }
    }

    for _depth in 0..max_depth {
        let mut next = Vec::new();
        for entry in frontier {
            if entry.path.len as usize >= max_depth || entry.path.len >= 12 {
                continue;
            }

            for (macro_index, macro_op) in macro_ops.iter().enumerate() {
                if entry.last_macro == Some(macro_index) {
                    continue;
                }

                let mut colors = entry.colors.clone();
                apply_macro_to_colors(puzzle, &mut colors, macro_op.move_index, macro_op.shift);
                *nodes += 1;
                if limits.exceeded(*nodes) {
                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        limits
                            .stop_reason
                            .clone()
                            .unwrap_or_else(|| "macro target table limit".to_string()),
                    ));
                }

                let key = state_key(&colors);
                if table.contains_key(&key) {
                    continue;
                }

                let path = entry.path.prepend(macro_index);
                let next_depth = path.len as usize;
                table.insert(key, path);
                depth_counts[next_depth] += 1;
                target_colors.push(colors.clone());
                next.push(MacroTargetEntry {
                    colors,
                    path,
                    last_macro: Some(macro_index),
                });
            }
        }
        frontier = next;
        if frontier.is_empty() {
            break;
        }
    }

    Ok((table, target_colors, depth_counts))
}

fn build_macro_target_projection_db(
    puzzle: &Puzzle,
    target_colors: &[Vec<Color>],
    projection: ProjectionKind,
    depth: usize,
    nodes: &mut u64,
    limits: &mut Limits,
) -> io::Result<PatternDb> {
    let mut distances = PatternDbDistances::new(projection);
    let mut seen = HashSet::new();
    let mut frontier = Vec::new();

    for colors in target_colors {
        let key = state_key(colors);
        if seen.insert(key) {
            distances.record(puzzle, colors, projection, 0);
            frontier.push(PatternEntry {
                colors: colors.clone(),
                last_move: None,
            });
        }
    }

    for current_depth in 0..depth {
        let mut next = Vec::new();
        let next_distance = (current_depth + 1).min(u8::MAX as usize) as u8;
        for entry in frontier {
            for move_index in 0..puzzle.moves.len() {
                if is_immediate_inverse(puzzle, entry.last_move, move_index) {
                    continue;
                }

                let mut colors = entry.colors.clone();
                puzzle.apply_move(&mut colors, move_index);
                *nodes += 1;
                if limits.exceeded(*nodes) {
                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        limits
                            .stop_reason
                            .clone()
                            .unwrap_or_else(|| "macro projection db limit".to_string()),
                    ));
                }

                let key = state_key(&colors);
                if !seen.insert(key) {
                    continue;
                }

                distances.record(puzzle, &colors, projection, next_distance);
                next.push(PatternEntry {
                    colors,
                    last_move: Some(move_index),
                });
            }
        }
        frontier = next;
        if frontier.is_empty() {
            break;
        }
    }

    Ok(PatternDb {
        distances,
        projection,
        use_canonical_fallback: false,
        depth,
        weight: DEFAULT_PATTERN_DB_WEIGHT,
    })
}

fn solve_macro_two_stage_hit(
    puzzle: &Puzzle,
    colors: &[Color],
    artifacts: &MacroTargetArtifacts,
    operations: &[Operation],
    config: &SolverConfig,
    require_suffix: bool,
) -> MacroTargetAttempt {
    let mut nodes = 0u64;
    let mut limits = Limits::new(config.max_nodes, config.time_limit_ms);

    if let Some(mut hit) = meet_macro_target(
        puzzle,
        colors,
        artifacts,
        config.forward_depth_for(puzzle.layout).min(12),
        require_suffix,
        &mut nodes,
        &mut limits,
    ) {
        hit.nodes = nodes;
        return MacroTargetAttempt {
            hit: Some(hit),
            nodes,
            reason: "found".to_string(),
        };
    }

    for &tier in &config.tiers {
        if limits.exceeded(nodes) {
            break;
        }
        if let Some(mut hit) = macro_target_beam(
            puzzle,
            colors,
            artifacts,
            operations,
            tier,
            config,
            require_suffix,
            &mut nodes,
            &mut limits,
        ) {
            hit.nodes = nodes;
            return MacroTargetAttempt {
                hit: Some(hit),
                nodes,
                reason: "found".to_string(),
            };
        }
    }

    MacroTargetAttempt {
        hit: None,
        nodes,
        reason: limits
            .stop_reason
            .clone()
            .unwrap_or_else(|| "not_found".to_string()),
    }
}

fn meet_macro_target(
    puzzle: &Puzzle,
    colors: &[Color],
    artifacts: &MacroTargetArtifacts,
    max_depth: usize,
    require_suffix: bool,
    nodes: &mut u64,
    limits: &mut Limits,
) -> Option<MacroTargetHit> {
    let start_key = state_key(colors);
    if let Some(&suffix_macro) = artifacts.table.get(&start_key) {
        if !require_suffix || suffix_macro.len > 0 {
            return Some(MacroTargetHit {
                prefix: Vec::new(),
                suffix_macro,
                nodes: *nodes,
                reason: "mitm:prefix_len=0".to_string(),
            });
        }
    }

    let mut seen = HashSet::new();
    seen.insert(start_key);
    let mut frontier = vec![ExactEntry {
        colors: colors.to_vec(),
        path: PathBits::default(),
        last_move: None,
    }];
    let mut best: Option<MacroTargetHit> = None;

    for _ in 0..max_depth {
        let mut next = Vec::new();
        for entry in frontier {
            if entry.path.len >= 12 {
                continue;
            }

            for move_index in 0..puzzle.moves.len() {
                if is_immediate_inverse(puzzle, entry.last_move, move_index) {
                    continue;
                }

                let mut next_colors = entry.colors.clone();
                puzzle.apply_move(&mut next_colors, move_index);
                *nodes += 1;
                if limits.exceeded(*nodes) {
                    return best;
                }

                let key = state_key(&next_colors);
                if !seen.insert(key) {
                    continue;
                }

                let prefix_bits = entry.path.append(move_index);
                if let Some(&suffix_macro) = artifacts.table.get(&key) {
                    if require_suffix && suffix_macro.len == 0 {
                        continue;
                    }
                    keep_best_macro_hit(
                        &mut best,
                        MacroTargetHit {
                            prefix: prefix_bits.to_vec(),
                            suffix_macro,
                            nodes: *nodes,
                            reason: format!("mitm:prefix_len={}", prefix_bits.len),
                        },
                        artifacts,
                    );
                    continue;
                }

                next.push(ExactEntry {
                    colors: next_colors,
                    path: prefix_bits,
                    last_move: Some(move_index),
                });
            }
        }
        frontier = next;
        if frontier.is_empty() {
            break;
        }
    }

    best
}

fn macro_target_beam(
    puzzle: &Puzzle,
    colors: &[Color],
    artifacts: &MacroTargetArtifacts,
    operations: &[Operation],
    tier: Tier,
    config: &SolverConfig,
    require_suffix: bool,
    nodes: &mut u64,
    limits: &mut Limits,
) -> Option<MacroTargetHit> {
    let seed_base = hash_colors(colors);
    let mut best: Option<MacroTargetHit> = None;

    for restart in 0..tier.restarts {
        let mut rng =
            Mulberry32::new(seed_base.wrapping_add((restart as u32).wrapping_mul(0x9e37_79b9)));
        let jitter = if restart == 0 {
            0
        } else {
            120 + restart as i32 * 60
        };
        let initial_score = macro_target_rank_score(puzzle, colors, 0, artifacts, config);
        let mut beam = vec![BeamEntry {
            colors: colors.to_vec(),
            path: Vec::new(),
            last_move: None,
            score: initial_score,
            rank_score: initial_score,
        }];
        let mut first_hit_depth: Option<usize> = None;

        for depth in 0..tier.max_depth {
            let mut candidates = Vec::new();
            let mut layer_seen = HashSet::new();

            for entry in &beam {
                for operation in operations {
                    if operation.is_raw
                        && entry
                            .last_move
                            .is_some_and(|last| puzzle.moves[operation.moves[0]].inverse == last)
                    {
                        continue;
                    }

                    let mut next_colors = entry.colors.clone();
                    for &move_index in &operation.moves {
                        puzzle.apply_move(&mut next_colors, move_index);
                    }
                    *nodes += 1;
                    if limits.exceeded(*nodes) {
                        return best;
                    }

                    let key = state_key(&next_colors);
                    if !layer_seen.insert(key) {
                        continue;
                    }

                    let mut path = entry.path.clone();
                    path.extend(&operation.path);

                    if let Some(&suffix_macro) = artifacts.table.get(&key) {
                        if require_suffix && suffix_macro.len == 0 {
                            continue;
                        }
                        first_hit_depth.get_or_insert(depth);
                        keep_best_macro_hit(
                            &mut best,
                            MacroTargetHit {
                                prefix: path,
                                suffix_macro,
                                nodes: *nodes,
                                reason: format!(
                                    "beam:restart={restart}:depth={depth}:prefix_len={}",
                                    entry.path.len() + operation.path.len()
                                ),
                            },
                            artifacts,
                        );
                        continue;
                    }

                    let score = macro_target_rank_score(
                        puzzle,
                        &next_colors,
                        path.len(),
                        artifacts,
                        config,
                    );
                    let rank_score = score + jitter_sample(&mut rng, jitter);
                    candidates.push(BeamEntry {
                        colors: next_colors,
                        path,
                        last_move: Some(operation.last_move),
                        score,
                        rank_score,
                    });
                }
            }

            if best.is_some()
                && first_hit_depth.is_some_and(|hit_depth| depth >= hit_depth + config.hit_patience)
            {
                return best;
            }

            if candidates.is_empty() {
                break;
            }
            candidates.sort_unstable_by(|left, right| {
                left.rank_score
                    .cmp(&right.rank_score)
                    .then_with(|| left.path.len().cmp(&right.path.len()))
                    .then_with(|| left.score.cmp(&right.score))
            });
            candidates.truncate(tier.width);
            beam = candidates;
        }
    }

    best
}

fn macro_target_rank_score(
    puzzle: &Puzzle,
    colors: &[Color],
    path_len: usize,
    artifacts: &MacroTargetArtifacts,
    config: &SolverConfig,
) -> i32 {
    let target_distance_score = artifacts.projection_db.as_ref().map_or(0, |pattern_db| {
        let distance = pattern_db
            .distance(colors, puzzle)
            .map(i32::from)
            .unwrap_or_else(|| pattern_db.depth as i32 + 1);
        distance * 100_000
    });
    let solved_score = score_state(colors, puzzle, TargetMode::Android, 0, None) / 10;
    target_distance_score + solved_score + path_len as i32 * config.path_penalty
}

fn keep_best_macro_hit(
    best: &mut Option<MacroTargetHit>,
    candidate: MacroTargetHit,
    artifacts: &MacroTargetArtifacts,
) {
    let candidate_len = macro_target_hit_unit_len(&candidate, artifacts);
    let current_len = best
        .as_ref()
        .map(|current| macro_target_hit_unit_len(current, artifacts))
        .unwrap_or(usize::MAX);
    if candidate_len < current_len {
        *best = Some(candidate);
    }
}

fn macro_target_hit_unit_len(hit: &MacroTargetHit, artifacts: &MacroTargetArtifacts) -> usize {
    hit.prefix.len() + macro_suffix_unit_len(&artifacts.macro_ops, hit.suffix_macro)
}

fn macro_suffix_unit_len(macro_ops: &[MacroOp], macro_path: PathBits) -> usize {
    macro_path
        .to_vec()
        .iter()
        .map(|&macro_index| macro_ops[macro_index].shift)
        .sum()
}

fn expand_macro_suffix(macro_ops: &[MacroOp], macro_path: PathBits) -> Vec<MoveIndex> {
    let mut out = Vec::with_capacity(macro_suffix_unit_len(macro_ops, macro_path));
    for macro_index in macro_path.to_vec() {
        let macro_op = &macro_ops[macro_index];
        for _ in 0..macro_op.shift {
            out.push(macro_op.move_index);
        }
    }
    out
}

fn restricted_seed_target(target_mode: TargetMode) -> TargetMode {
    match target_mode {
        TargetMode::AndroidPortfolio => TargetMode::AndroidMultiGoal,
        TargetMode::PairRegion => TargetMode::Android,
        other => other,
    }
}

fn build_restricted_target_artifacts(
    puzzle: &Puzzle,
    target_mode: TargetMode,
    allowed_moves: Vec<MoveIndex>,
    max_depth: usize,
    pattern_depth: usize,
    config: &SolverConfig,
) -> io::Result<RestrictedTargetArtifacts> {
    let started = Instant::now();
    let mut build_nodes = 0u64;
    let mut build_limits = Limits::new(config.max_nodes, config.time_limit_ms.max(120_000));
    let (mut table, mut target_colors, mut depth_counts) = build_restricted_color_table(
        puzzle,
        target_mode,
        &allowed_moves,
        max_depth,
        &mut build_nodes,
        &mut build_limits,
    )?;
    if config.target_expand_depth > 0 {
        let expansion_moves = (0..puzzle.moves.len()).collect::<Vec<_>>();
        expand_restricted_target_table(
            puzzle,
            &mut table,
            &mut target_colors,
            &mut depth_counts,
            &expansion_moves,
            config.target_expand_depth,
            &mut build_nodes,
            &mut build_limits,
        )?;
    }
    let build_ms = started.elapsed().as_millis();

    let projection_started = Instant::now();
    let mut projection_build_nodes = 0u64;
    let projection_db = if pattern_depth > 0 {
        let mut projection_limits =
            Limits::new(config.max_nodes, config.time_limit_ms.max(120_000));
        Some(build_macro_target_projection_db(
            puzzle,
            &target_colors,
            config.pattern_db_projection,
            pattern_depth,
            &mut projection_build_nodes,
            &mut projection_limits,
        )?)
    } else {
        None
    };
    let projection_build_ms = projection_started.elapsed().as_millis();
    let axis_hint = restricted_target_axis_hint(puzzle, &allowed_moves);
    let axis_ring_profiles = axis_hint.map_or_else(Vec::new, |axis| {
        build_axis_ring_profiles(puzzle, target_mode, axis)
    });

    Ok(RestrictedTargetArtifacts {
        table,
        target_colors,
        allowed_moves,
        depth_counts,
        build_nodes,
        build_ms,
        projection_db,
        projection_build_nodes,
        projection_build_ms,
        axis_hint,
        axis_ring_profiles,
    })
}

fn build_restricted_color_table(
    puzzle: &Puzzle,
    target_mode: TargetMode,
    allowed_moves: &[MoveIndex],
    max_depth: usize,
    nodes: &mut u64,
    limits: &mut Limits,
) -> io::Result<(HashMap<Key, Vec<MoveIndex>>, Vec<Vec<Color>>, Vec<usize>)> {
    let mut table: HashMap<Key, Vec<MoveIndex>> = HashMap::new();
    let mut target_colors = Vec::new();
    let mut depth_counts = vec![0usize; max_depth + 1];
    let mut frontier = Vec::new();

    for seed in reverse_table_seeds(puzzle, target_mode) {
        let key = state_key(&seed);
        if table.insert(key, Vec::new()).is_none() {
            depth_counts[0] += 1;
            target_colors.push(seed.clone());
            frontier.push(RestrictedTargetEntry {
                colors: seed,
                suffix: Vec::new(),
                last_move: None,
            });
        }
    }

    for _depth in 0..max_depth {
        let mut next = Vec::new();
        for entry in frontier {
            if entry.suffix.len() >= max_depth {
                continue;
            }

            for &move_index in allowed_moves {
                if is_immediate_inverse(puzzle, entry.last_move, move_index) {
                    continue;
                }

                let mut colors = entry.colors.clone();
                puzzle.apply_move(&mut colors, move_index);
                *nodes += 1;
                if limits.exceeded(*nodes) {
                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        limits
                            .stop_reason
                            .clone()
                            .unwrap_or_else(|| "restricted target table limit".to_string()),
                    ));
                }

                let key = state_key(&colors);
                if table.contains_key(&key) {
                    continue;
                }

                let mut suffix = Vec::with_capacity(entry.suffix.len() + 1);
                suffix.push(puzzle.inverse_index(move_index));
                suffix.extend(entry.suffix.iter().copied());
                let next_depth = suffix.len();
                table.insert(key, suffix.clone());
                depth_counts[next_depth] += 1;
                target_colors.push(colors.clone());
                next.push(RestrictedTargetEntry {
                    colors,
                    suffix,
                    last_move: Some(move_index),
                });
            }
        }
        frontier = next;
        if frontier.is_empty() {
            break;
        }
    }

    Ok((table, target_colors, depth_counts))
}

fn expand_restricted_target_table(
    puzzle: &Puzzle,
    table: &mut HashMap<Key, Vec<MoveIndex>>,
    target_colors: &mut Vec<Vec<Color>>,
    depth_counts: &mut Vec<usize>,
    expand_moves: &[MoveIndex],
    expand_depth: usize,
    nodes: &mut u64,
    limits: &mut Limits,
) -> io::Result<()> {
    if expand_depth == 0 || expand_moves.is_empty() {
        return Ok(());
    }

    let mut frontier = target_colors
        .iter()
        .filter_map(|colors| {
            table
                .get(&state_key(colors))
                .cloned()
                .map(|suffix| RestrictedTargetEntry {
                    colors: colors.clone(),
                    suffix,
                    last_move: None,
                })
        })
        .collect::<Vec<_>>();

    for _ in 0..expand_depth {
        let mut next = Vec::new();
        for entry in frontier {
            for &move_index in expand_moves {
                if is_immediate_inverse(puzzle, entry.last_move, move_index) {
                    continue;
                }

                let mut colors = entry.colors.clone();
                puzzle.apply_move(&mut colors, move_index);
                *nodes += 1;
                if limits.exceeded(*nodes) {
                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        limits
                            .stop_reason
                            .clone()
                            .unwrap_or_else(|| "restricted target expansion limit".to_string()),
                    ));
                }

                let key = state_key(&colors);
                if table.contains_key(&key) {
                    continue;
                }

                let mut suffix = Vec::with_capacity(entry.suffix.len() + 1);
                suffix.push(puzzle.inverse_index(move_index));
                suffix.extend(entry.suffix.iter().copied());
                let suffix_depth = suffix.len();
                if suffix_depth >= depth_counts.len() {
                    depth_counts.resize(suffix_depth + 1, 0);
                }
                table.insert(key, suffix.clone());
                depth_counts[suffix_depth] += 1;
                target_colors.push(colors.clone());
                next.push(RestrictedTargetEntry {
                    colors,
                    suffix,
                    last_move: Some(move_index),
                });
            }
        }
        frontier = next;
        if frontier.is_empty() {
            break;
        }
    }

    Ok(())
}

fn axis_ring_rescue_config(options: &BenchOptions) -> SolverConfig {
    let mut config = options.solver.clone();
    config.time_limit_ms = options.axis_ring_rescue_time_limit_ms;
    config.tiers = vec![options.axis_ring_rescue_tier];
    config.target_expand_depth = options.axis_ring_rescue_expand_depth;
    config.axis_ring_profile_weight = 0;
    config.axis_ring_order_weight = 0;
    config
}

fn build_axis_ring_rescue_artifacts(
    puzzle: &Puzzle,
    options: &BenchOptions,
) -> io::Result<AxisRingRescueArtifacts> {
    let target_mode = restricted_seed_target(options.solver.target_mode);
    let config = axis_ring_rescue_config(options);
    let operation_profile = options.solver.operation_profile.for_layout(puzzle.layout);
    let operations = build_operations(puzzle, operation_profile);
    let mut axes = Vec::new();

    for axis in 0..3_u8 {
        let moves = axis_moves(puzzle, axis);
        if moves.is_empty() {
            continue;
        }
        let artifacts = build_restricted_target_artifacts(
            puzzle,
            target_mode,
            moves,
            options.axis_ring_rescue_table_depth,
            options.axis_ring_rescue_pattern_depth,
            &config,
        )?;
        axes.push((axis, artifacts));
    }

    Ok(AxisRingRescueArtifacts {
        target_mode,
        axes,
        operations,
        operation_profile,
    })
}

fn should_run_axis_ring_rescue(
    options: &BenchOptions,
    puzzle: &Puzzle,
    cascade_found: bool,
    cascade_opt_len: usize,
) -> bool {
    options.axis_ring_rescue_enabled
        && options.e_classic_cascade
        && puzzle.layout == LayoutId::E
        && puzzle.difficulty == Difficulty::Classic
        && (!cascade_found || cascade_opt_len >= options.axis_ring_rescue_threshold)
}

fn should_skip_corner_after_axis_ring(options: &BenchOptions, cascade_opt_len: usize) -> bool {
    options.axis_ring_rescue_position.runs_before_corner()
        && options.axis_ring_rescue_corner_skip_threshold > 0
        && cascade_opt_len <= options.axis_ring_rescue_corner_skip_threshold
}

fn run_axis_ring_rescue_record(
    puzzle: &Puzzle,
    options: &BenchOptions,
    cache: &mut Option<AxisRingRescueArtifacts>,
    solver_artifacts: &SolverArtifacts,
    layout: LayoutId,
    difficulty: Difficulty,
    scramble_len: usize,
    iteration: usize,
    seed: u64,
    ariadne_solution_len: usize,
    direct: &SolveResult,
    cascade_found: bool,
    cascade_opt_len: usize,
    scrambled_colors: &[Color],
) -> io::Result<PhaseLabRecord> {
    if cache.is_none() {
        let built = build_axis_ring_rescue_artifacts(puzzle, options)?;
        println!(
            "prepared {layout}-{difficulty} axis_ring_rescue axes={} states={} expand={} threshold={} position={}",
            built.axes.len(),
            built
                .axes
                .iter()
                .map(|(_, artifacts)| artifacts.table.len())
                .sum::<usize>(),
            options.axis_ring_rescue_expand_depth,
            options.axis_ring_rescue_threshold,
            options.axis_ring_rescue_position.label()
        );
        *cache = Some(built);
    }

    let axis_artifacts = cache.as_ref().expect("axis ring rescue artifacts");
    let axis_result = solve_axis_ring_rescue(
        puzzle,
        scrambled_colors,
        axis_artifacts,
        solver_artifacts,
        options,
        options.macro_require_suffix,
    );
    Ok(axis_ring_rescue_phase_record(
        options,
        layout,
        difficulty,
        scramble_len,
        iteration,
        seed,
        ariadne_solution_len,
        direct,
        cascade_found,
        cascade_opt_len,
        axis_result,
    ))
}

fn optimizer_table_for_target<'a>(
    artifacts: &'a SolverArtifacts,
    target_mode: TargetMode,
) -> Option<&'a HashMap<Key, PathBits>> {
    artifacts
        .variants
        .iter()
        .find(|variant| variant.target_mode == target_mode)
        .or_else(|| artifacts.variants.first())
        .map(|variant| &variant.table)
}

fn solve_axis_ring_rescue(
    puzzle: &Puzzle,
    colors: &[Color],
    rescue_artifacts: &AxisRingRescueArtifacts,
    solver_artifacts: &SolverArtifacts,
    options: &BenchOptions,
    require_suffix: bool,
) -> AxisRingRescueResult {
    let started = Instant::now();
    let mut config = axis_ring_rescue_config(options);
    config.operation_profile = rescue_artifacts.operation_profile;
    let optimizer_table =
        optimizer_table_for_target(solver_artifacts, rescue_artifacts.target_mode);
    let mut total_nodes = 0u64;
    let mut best: Option<AxisRingRescueResult> = None;
    let mut failure_reasons = BTreeMap::new();

    for (axis, artifacts) in &rescue_artifacts.axes {
        let attempt = solve_restricted_target_hit(
            puzzle,
            colors,
            artifacts,
            &rescue_artifacts.operations,
            &config,
            require_suffix,
        );
        total_nodes += attempt.nodes;

        let Some(hit) = attempt.hit else {
            *failure_reasons.entry(attempt.reason).or_insert(0usize) += 1;
            continue;
        };

        let prefix_len = hit.prefix.len();
        let suffix_len = hit.suffix.len();
        let mut raw_moves = hit.prefix;
        raw_moves.extend(hit.suffix);
        let optimized_moves = if config.optimize {
            if let Some(table) = optimizer_table {
                let (local_window, local_depth) = local_optimization_for_profile(
                    puzzle,
                    &config,
                    rescue_artifacts.operation_profile,
                );
                optimize_solution_with_table(
                    puzzle,
                    colors,
                    &raw_moves,
                    table,
                    local_window,
                    local_depth,
                )
            } else {
                optimize_solution(puzzle, colors, &raw_moves)
            }
        } else {
            raw_moves.clone()
        };

        let found = solution_matches_target(
            puzzle,
            colors,
            &optimized_moves,
            acceptance_target(rescue_artifacts.target_mode),
        );
        let reason = if found {
            hit.reason
        } else {
            format!("{}:invalid_target", hit.reason)
        };
        let candidate = AxisRingRescueResult {
            found,
            axis: Some(*axis),
            reason,
            prefix_len,
            suffix_len,
            raw_moves,
            optimized_moves,
            nodes: total_nodes,
            elapsed_ms: started.elapsed().as_millis(),
        };

        if candidate.found {
            keep_best_axis_ring_rescue(&mut best, candidate);
        } else {
            *failure_reasons.entry(candidate.reason).or_insert(0usize) += 1;
        }
    }

    if let Some(mut best) = best {
        best.nodes = total_nodes;
        best.elapsed_ms = started.elapsed().as_millis();
        return best;
    }

    AxisRingRescueResult {
        found: false,
        axis: None,
        reason: if failure_reasons.is_empty() {
            "axis-ring:not_found".to_string()
        } else {
            join_counts(&failure_reasons)
        },
        prefix_len: 0,
        suffix_len: 0,
        raw_moves: Vec::new(),
        optimized_moves: Vec::new(),
        nodes: total_nodes,
        elapsed_ms: started.elapsed().as_millis(),
    }
}

fn keep_best_axis_ring_rescue(
    best: &mut Option<AxisRingRescueResult>,
    candidate: AxisRingRescueResult,
) {
    let candidate_key = (
        !candidate.found,
        candidate.optimized_moves.len(),
        candidate.raw_moves.len(),
        candidate.nodes,
    );
    let current_key = best
        .as_ref()
        .map(|current| {
            (
                !current.found,
                current.optimized_moves.len(),
                current.raw_moves.len(),
                current.nodes,
            )
        })
        .unwrap_or((true, usize::MAX, usize::MAX, u64::MAX));
    if candidate_key < current_key {
        *best = Some(candidate);
    }
}

fn axis_ring_rescue_phase_record(
    options: &BenchOptions,
    layout: LayoutId,
    difficulty: Difficulty,
    scramble_len: usize,
    iteration: usize,
    seed: u64,
    ariadne_solution_len: usize,
    direct: &SolveResult,
    cascade_found: bool,
    cascade_opt_len: usize,
    result: AxisRingRescueResult,
) -> PhaseLabRecord {
    let direct_opt_len = direct.optimized_moves.len();
    let total_opt_len = result.optimized_moves.len();
    let total_raw_len = result.raw_moves.len();
    let axis_label = result.axis.map(axis_label).unwrap_or("none");
    let total_found = result.found;
    let delta_opt_vs_direct = if total_found && direct.found {
        total_opt_len as isize - direct_opt_len as isize
    } else {
        0
    };
    let candidate_min_prefix_len = if total_found { result.prefix_len } else { 0 };
    let candidate_prefix_lens = if total_found {
        result.prefix_len.to_string()
    } else {
        String::new()
    };
    let reason = format!(
        "axis={axis_label}; gate_threshold={}; cascade_found={}; cascade_opt={}; {}",
        options.axis_ring_rescue_threshold,
        cascade_found,
        if cascade_found {
            cascade_opt_len.to_string()
        } else {
            "-".to_string()
        },
        result.reason
    );

    PhaseLabRecord {
        layout,
        difficulty,
        scramble_len,
        iteration: iteration + 1,
        seed,
        ariadne_solution_len,
        phase_kind: "axis-ring-rescue".to_string(),
        phase_spec: format!(
            "axis-ring-rescue:position={}:target={}:expand={}:table_depth={}:pattern_depth={}:threshold={}",
            options.axis_ring_rescue_position.label(),
            restricted_seed_target(options.solver.target_mode).label(),
            options.axis_ring_rescue_expand_depth,
            options.axis_ring_rescue_table_depth,
            options.axis_ring_rescue_pattern_depth,
            options.axis_ring_rescue_threshold
        ),
        direct_found: direct.found,
        direct_opt_len,
        direct_elapsed_ms: direct.elapsed_ms,
        phase_found: total_found,
        suffix_found: total_found,
        total_found,
        prefix_len: result.prefix_len,
        suffix_len: result.suffix_len,
        total_raw_len,
        total_opt_len,
        delta_opt_vs_direct,
        phase_elapsed_ms: 0,
        suffix_elapsed_ms: result.elapsed_ms,
        total_elapsed_ms: result.elapsed_ms,
        nodes: result.nodes,
        prefixes_available: 3,
        prefixes_tested: 3,
        candidate_min_prefix_len,
        candidate_prefix_lens,
        candidate_signatures: String::new(),
        reason,
    }
}

fn audit_restricted_subgroup(
    puzzle: &Puzzle,
    allowed_moves: &[MoveIndex],
    max_depth: usize,
    max_nodes: u64,
) -> MacroSubgroupAuditResult {
    let started = Instant::now();
    let identity_permutation = (0..puzzle.stickers.len())
        .map(|index| index as u8)
        .collect::<Vec<_>>();
    let mut permutation_seen = HashSet::new();
    let mut color_seen = HashSet::new();
    let mut depth_counts = vec![0usize; max_depth + 1];
    let mut color_depth_counts = vec![0usize; max_depth + 1];
    let mut queue = VecDeque::new();

    permutation_seen.insert(identity_permutation.clone());
    color_seen.insert(state_key(&puzzle.solved_colors));
    depth_counts[0] = 1;
    color_depth_counts[0] = 1;
    queue.push_back(MacroSubgroupNode {
        permutation: identity_permutation,
        colors: puzzle.solved_colors.clone(),
        depth: 0,
        last_macro: None,
    });

    let mut explored = 0u64;
    let mut truncated = false;
    let mut depth_limited = false;

    while let Some(node) = queue.pop_front() {
        if explored >= max_nodes {
            truncated = true;
            break;
        }
        explored += 1;
        if node.depth >= max_depth {
            depth_limited = true;
            continue;
        }

        for (move_pos, &move_index) in allowed_moves.iter().enumerate() {
            if node
                .last_macro
                .is_some_and(|last| allowed_moves[last] == puzzle.inverse_index(move_index))
            {
                continue;
            }

            let mut next_permutation = node.permutation.clone();
            apply_move_to_permutation(puzzle, &mut next_permutation, move_index);

            let mut next_colors = node.colors.clone();
            puzzle.apply_move(&mut next_colors, move_index);

            let next_depth = node.depth + 1;
            let is_new_permutation = permutation_seen.insert(next_permutation.clone());
            if color_seen.insert(state_key(&next_colors)) {
                color_depth_counts[next_depth] += 1;
            }

            if is_new_permutation {
                depth_counts[next_depth] += 1;
                queue.push_back(MacroSubgroupNode {
                    permutation: next_permutation,
                    colors: next_colors,
                    depth: next_depth,
                    last_macro: Some(move_pos),
                });
            }
        }
    }

    MacroSubgroupAuditResult {
        shift: 1,
        macro_count: allowed_moves.len(),
        explored,
        permutation_states: permutation_seen.len(),
        color_states: color_seen.len(),
        depth_counts,
        color_depth_counts,
        exhausted: queue.is_empty() && !truncated && !depth_limited,
        depth_limited,
        truncated,
        elapsed_ms: started.elapsed().as_millis(),
    }
}

fn solve_restricted_target_hit(
    puzzle: &Puzzle,
    colors: &[Color],
    artifacts: &RestrictedTargetArtifacts,
    operations: &[Operation],
    config: &SolverConfig,
    require_suffix: bool,
) -> RestrictedTargetAttempt {
    let mut nodes = 0u64;
    let mut limits = Limits::new(config.max_nodes, config.time_limit_ms);

    if let Some(mut hit) = meet_restricted_target(
        puzzle,
        colors,
        artifacts,
        config.forward_depth_for(puzzle.layout).min(12),
        require_suffix,
        &mut nodes,
        &mut limits,
    ) {
        hit.nodes = nodes;
        return RestrictedTargetAttempt {
            hit: Some(hit),
            nodes,
            reason: "found".to_string(),
        };
    }

    for &tier in &config.tiers {
        if limits.exceeded(nodes) {
            break;
        }
        if let Some(mut hit) = restricted_target_beam(
            puzzle,
            colors,
            artifacts,
            operations,
            tier,
            config,
            require_suffix,
            &mut nodes,
            &mut limits,
        ) {
            hit.nodes = nodes;
            return RestrictedTargetAttempt {
                hit: Some(hit),
                nodes,
                reason: "found".to_string(),
            };
        }
    }

    RestrictedTargetAttempt {
        hit: None,
        nodes,
        reason: limits
            .stop_reason
            .clone()
            .unwrap_or_else(|| "not_found".to_string()),
    }
}

fn meet_restricted_target(
    puzzle: &Puzzle,
    colors: &[Color],
    artifacts: &RestrictedTargetArtifacts,
    max_depth: usize,
    require_suffix: bool,
    nodes: &mut u64,
    limits: &mut Limits,
) -> Option<RestrictedTargetHit> {
    let start_key = state_key(colors);
    if let Some(suffix) = artifacts.table.get(&start_key) {
        if !require_suffix || !suffix.is_empty() {
            return Some(RestrictedTargetHit {
                prefix: Vec::new(),
                suffix: suffix.clone(),
                nodes: *nodes,
                reason: "mitm:prefix_len=0".to_string(),
            });
        }
    }

    let mut seen = HashSet::new();
    seen.insert(start_key);
    let mut frontier = vec![ExactEntry {
        colors: colors.to_vec(),
        path: PathBits::default(),
        last_move: None,
    }];
    let mut best: Option<RestrictedTargetHit> = None;

    for _ in 0..max_depth {
        let mut next = Vec::new();
        for entry in frontier {
            if entry.path.len >= 12 {
                continue;
            }

            for move_index in 0..puzzle.moves.len() {
                if is_immediate_inverse(puzzle, entry.last_move, move_index) {
                    continue;
                }

                let mut next_colors = entry.colors.clone();
                puzzle.apply_move(&mut next_colors, move_index);
                *nodes += 1;
                if limits.exceeded(*nodes) {
                    return best;
                }

                let key = state_key(&next_colors);
                if !seen.insert(key) {
                    continue;
                }

                let prefix_bits = entry.path.append(move_index);
                if let Some(suffix) = artifacts.table.get(&key) {
                    if require_suffix && suffix.is_empty() {
                        continue;
                    }
                    keep_best_restricted_hit(
                        &mut best,
                        RestrictedTargetHit {
                            prefix: prefix_bits.to_vec(),
                            suffix: suffix.clone(),
                            nodes: *nodes,
                            reason: format!("mitm:prefix_len={}", prefix_bits.len),
                        },
                    );
                    continue;
                }

                next.push(ExactEntry {
                    colors: next_colors,
                    path: prefix_bits,
                    last_move: Some(move_index),
                });
            }
        }
        frontier = next;
        if frontier.is_empty() {
            break;
        }
    }

    best
}

fn restricted_target_beam(
    puzzle: &Puzzle,
    colors: &[Color],
    artifacts: &RestrictedTargetArtifacts,
    operations: &[Operation],
    tier: Tier,
    config: &SolverConfig,
    require_suffix: bool,
    nodes: &mut u64,
    limits: &mut Limits,
) -> Option<RestrictedTargetHit> {
    let seed_base = hash_colors(colors);
    let mut best: Option<RestrictedTargetHit> = None;

    for restart in 0..tier.restarts {
        let mut rng =
            Mulberry32::new(seed_base.wrapping_add((restart as u32).wrapping_mul(0x9e37_79b9)));
        let jitter = if restart == 0 {
            0
        } else {
            120 + restart as i32 * 60
        };
        let initial_score = restricted_target_rank_score(puzzle, colors, 0, artifacts, config);
        let mut beam = vec![BeamEntry {
            colors: colors.to_vec(),
            path: Vec::new(),
            last_move: None,
            score: initial_score,
            rank_score: initial_score,
        }];
        let mut first_hit_depth: Option<usize> = None;

        for depth in 0..tier.max_depth {
            let mut candidates = Vec::new();
            let mut layer_seen = HashSet::new();

            for entry in &beam {
                for operation in operations {
                    if operation.is_raw
                        && entry
                            .last_move
                            .is_some_and(|last| puzzle.moves[operation.moves[0]].inverse == last)
                    {
                        continue;
                    }

                    let mut next_colors = entry.colors.clone();
                    for &move_index in &operation.moves {
                        puzzle.apply_move(&mut next_colors, move_index);
                    }
                    *nodes += 1;
                    if limits.exceeded(*nodes) {
                        return best;
                    }

                    let key = state_key(&next_colors);
                    if !layer_seen.insert(key) {
                        continue;
                    }

                    let mut path = entry.path.clone();
                    path.extend(&operation.path);

                    if let Some(suffix) = artifacts.table.get(&key) {
                        if require_suffix && suffix.is_empty() {
                            continue;
                        }
                        first_hit_depth.get_or_insert(depth);
                        keep_best_restricted_hit(
                            &mut best,
                            RestrictedTargetHit {
                                prefix: path,
                                suffix: suffix.clone(),
                                nodes: *nodes,
                                reason: format!(
                                    "beam:restart={restart}:depth={depth}:prefix_len={}",
                                    entry.path.len() + operation.path.len()
                                ),
                            },
                        );
                        continue;
                    }

                    let score = restricted_target_rank_score(
                        puzzle,
                        &next_colors,
                        path.len(),
                        artifacts,
                        config,
                    );
                    let rank_score = score + jitter_sample(&mut rng, jitter);
                    candidates.push(BeamEntry {
                        colors: next_colors,
                        path,
                        last_move: Some(operation.last_move),
                        score,
                        rank_score,
                    });
                }
            }

            if best.is_some()
                && first_hit_depth.is_some_and(|hit_depth| depth >= hit_depth + config.hit_patience)
            {
                return best;
            }

            if candidates.is_empty() {
                break;
            }
            candidates.sort_unstable_by(|left, right| {
                left.rank_score
                    .cmp(&right.rank_score)
                    .then_with(|| left.path.len().cmp(&right.path.len()))
                    .then_with(|| left.score.cmp(&right.score))
            });
            candidates.truncate(tier.width);
            beam = candidates;
        }
    }

    best
}

fn restricted_target_rank_score(
    puzzle: &Puzzle,
    colors: &[Color],
    path_len: usize,
    artifacts: &RestrictedTargetArtifacts,
    config: &SolverConfig,
) -> i32 {
    let target_distance_score = artifacts.projection_db.as_ref().map_or(0, |pattern_db| {
        let distance = pattern_db
            .distance(colors, puzzle)
            .map(i32::from)
            .unwrap_or_else(|| pattern_db.depth as i32 + 1);
        distance * 100_000
    });
    let axis_score = if config.axis_ring_profile_weight > 0 {
        artifacts.axis_hint.map_or(0, |axis| {
            axis_ring_profile_score(puzzle, colors, axis, &artifacts.axis_ring_profiles)
                * config.axis_ring_profile_weight
        })
    } else {
        0
    };
    let axis_order_score = if config.axis_ring_order_weight > 0 {
        artifacts.axis_hint.map_or(0, |axis| {
            axis_ring_order_score(puzzle, colors, axis, &artifacts.axis_ring_profiles)
                * config.axis_ring_order_weight
        })
    } else {
        0
    };
    let solved_score = score_state(colors, puzzle, TargetMode::Android, 0, None) / 10;
    target_distance_score
        + axis_score
        + axis_order_score
        + solved_score
        + path_len as i32 * config.path_penalty
}

fn keep_best_restricted_hit(
    best: &mut Option<RestrictedTargetHit>,
    candidate: RestrictedTargetHit,
) {
    let candidate_len = candidate.prefix.len() + candidate.suffix.len();
    let current_len = best
        .as_ref()
        .map(|current| current.prefix.len() + current.suffix.len())
        .unwrap_or(usize::MAX);
    if candidate_len < current_len {
        *best = Some(candidate);
    }
}

fn keep_best_axis_ring_record(best: &mut Option<AxisRingRecord>, candidate: AxisRingRecord) {
    let candidate_key = (
        !candidate.found,
        candidate.optimized_solution_len,
        candidate.raw_solution_len,
        candidate.nodes,
    );
    let current_key = best
        .as_ref()
        .map(|current| {
            (
                !current.found,
                current.optimized_solution_len,
                current.raw_solution_len,
                current.nodes,
            )
        })
        .unwrap_or((true, usize::MAX, usize::MAX, u64::MAX));
    if candidate_key < current_key {
        *best = Some(candidate);
    }
}

fn axis_moves(puzzle: &Puzzle, axis: u8) -> Vec<MoveIndex> {
    puzzle
        .moves
        .iter()
        .enumerate()
        .filter_map(|(move_index, mv)| (mv.axis == axis).then_some(move_index))
        .collect()
}

fn restricted_target_axis_hint(puzzle: &Puzzle, allowed_moves: &[MoveIndex]) -> Option<u8> {
    let first = *allowed_moves.first()?;
    let axis = puzzle.moves[first].axis;
    if !allowed_moves
        .iter()
        .all(|&move_index| puzzle.moves[move_index].axis == axis)
    {
        return None;
    }

    let expected = axis_layer_count(puzzle.layout, axis) * 2;
    (allowed_moves.len() == expected).then_some(axis)
}

fn build_axis_ring_profiles(
    puzzle: &Puzzle,
    target_mode: TargetMode,
    axis: u8,
) -> Vec<AxisRingProfile> {
    let ring_cycles = axis_ring_cycles(puzzle, axis);
    let (fixed_left, fixed_right) = axis_fixed_faces(axis);
    let mut profiles = Vec::new();
    let mut seen = HashSet::new();

    for seed in reverse_table_seeds(puzzle, target_mode) {
        let fixed_colors = (
            seed[puzzle.face_indexes[fixed_left.index()][0]],
            seed[puzzle.face_indexes[fixed_right.index()][0]],
        );
        let ring_counts = ring_cycles
            .iter()
            .map(|cycle| {
                let mut counts = [0usize; 6];
                for &index in *cycle {
                    counts[seed[index] as usize] += 1;
                }
                counts
            })
            .collect::<Vec<_>>();
        let ring_sequences = ring_cycles
            .iter()
            .map(|cycle| cycle.iter().map(|&index| seed[index]).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let mut key = vec![fixed_colors.0, fixed_colors.1];
        for sequence in &ring_sequences {
            key.extend(sequence);
        }
        if seen.insert(key) {
            profiles.push(AxisRingProfile {
                fixed_colors,
                ring_counts,
                ring_sequences,
            });
        }
    }

    profiles
}

fn axis_ring_profile_score(
    puzzle: &Puzzle,
    colors: &[Color],
    axis: u8,
    profiles: &[AxisRingProfile],
) -> i32 {
    if profiles.is_empty() {
        return 0;
    }

    let (fixed_left, fixed_right) = axis_fixed_faces(axis);
    let left_indexes = &puzzle.face_indexes[fixed_left.index()];
    let right_indexes = &puzzle.face_indexes[fixed_right.index()];
    let ring_cycles = axis_ring_cycles(puzzle, axis);
    let current_ring_counts = ring_cycles
        .iter()
        .map(|cycle| {
            let mut counts = [0usize; 6];
            for &index in *cycle {
                counts[colors[index] as usize] += 1;
            }
            counts
        })
        .collect::<Vec<_>>();

    profiles
        .iter()
        .map(|profile| {
            let fixed_misses = left_indexes
                .iter()
                .filter(|&&index| colors[index] != profile.fixed_colors.0)
                .count()
                + right_indexes
                    .iter()
                    .filter(|&&index| colors[index] != profile.fixed_colors.1)
                    .count();
            let ring_misses = current_ring_counts
                .iter()
                .zip(&profile.ring_counts)
                .map(|(current, target)| {
                    current
                        .iter()
                        .zip(target)
                        .map(|(&left, &right)| left.abs_diff(right))
                        .sum::<usize>()
                        / 2
                })
                .sum::<usize>();
            (fixed_misses + ring_misses) as i32
        })
        .min()
        .unwrap_or_default()
}

fn axis_ring_order_score(
    puzzle: &Puzzle,
    colors: &[Color],
    axis: u8,
    profiles: &[AxisRingProfile],
) -> i32 {
    if profiles.is_empty() {
        return 0;
    }

    let (fixed_left, fixed_right) = axis_fixed_faces(axis);
    let left_indexes = &puzzle.face_indexes[fixed_left.index()];
    let right_indexes = &puzzle.face_indexes[fixed_right.index()];
    let ring_cycles = axis_ring_cycles(puzzle, axis);
    let current_sequences = ring_cycles
        .iter()
        .map(|cycle| cycle.iter().map(|&index| colors[index]).collect::<Vec<_>>())
        .collect::<Vec<_>>();

    profiles
        .iter()
        .map(|profile| {
            let fixed_misses = left_indexes
                .iter()
                .filter(|&&index| colors[index] != profile.fixed_colors.0)
                .count()
                + right_indexes
                    .iter()
                    .filter(|&&index| colors[index] != profile.fixed_colors.1)
                    .count();
            let ring_misses = current_sequences
                .iter()
                .zip(&profile.ring_sequences)
                .map(|(current, target)| cyclic_hamming_misses(current, target))
                .sum::<usize>();
            (fixed_misses + ring_misses) as i32
        })
        .min()
        .unwrap_or_default()
}

fn cyclic_hamming_misses(current: &[Color], target: &[Color]) -> usize {
    if current.len() != target.len() || current.is_empty() {
        return current.len().max(target.len());
    }

    (0..current.len())
        .map(|shift| {
            current
                .iter()
                .enumerate()
                .filter(|&(index, &color)| color != target[(index + shift) % target.len()])
                .count()
        })
        .min()
        .unwrap_or_default()
}

fn axis_ring_cycles(puzzle: &Puzzle, axis: u8) -> Vec<&[usize]> {
    puzzle
        .moves
        .iter()
        .filter(|mv| mv.axis == axis && mv.direction > 0)
        .map(|mv| mv.cycle.as_slice())
        .collect()
}

fn middle_axis_moves(puzzle: &Puzzle) -> Vec<MoveIndex> {
    puzzle
        .moves
        .iter()
        .enumerate()
        .filter_map(|(move_index, mv)| {
            let layer_count = axis_layer_count(puzzle.layout, mv.axis);
            (layer_count > 0 && mv.layer == layer_count / 2).then_some(move_index)
        })
        .collect()
}

fn axis_layer_count(layout: LayoutId, axis: u8) -> usize {
    let dims = layout.dims();
    match axis {
        0 => dims.rows,
        1 => dims.cols,
        2 => dims.layers,
        _ => 0,
    }
}

fn axis_label(axis: u8) -> &'static str {
    match axis {
        0 => "x",
        1 => "y",
        2 => "z",
        _ => "?",
    }
}

fn axis_fixed_faces(axis: u8) -> (Face, Face) {
    match axis {
        0 => (Face::Left, Face::Right),
        1 => (Face::Top, Face::Bottom),
        2 => (Face::Front, Face::Back),
        _ => (Face::Front, Face::Back),
    }
}

fn axis_fixed_faces_label(axis: u8) -> String {
    let (left, right) = axis_fixed_faces(axis);
    format!("{}+{}", left.name(), right.name())
}

fn update_phase_vector(
    puzzle: &Puzzle,
    tapes: &[TapeCoord],
    phase: u64,
    move_index: MoveIndex,
    modulus: u8,
) -> u64 {
    let mv = &puzzle.moves[move_index];
    let slot = tapes
        .iter()
        .position(|tape| tape.axis == mv.axis && tape.layer == mv.layer)
        .expect("move tape must be present in tape coordinate list");
    let shift = slot * 4;
    let mask = 0xF_u64 << shift;
    let current = ((phase & mask) >> shift) as u8;
    let delta = if mv.direction > 0 { 1 } else { modulus - 1 };
    let next = (current + delta) % modulus;
    (phase & !mask) | ((next as u64) << shift)
}

fn phase_vector_values(tape_count: usize, phase: u64) -> Vec<u8> {
    (0..tape_count)
        .map(|slot| ((phase >> (slot * 4)) & 0xF) as u8)
        .collect()
}

fn format_phase_vector(tapes: &[TapeCoord], phase: u64) -> String {
    tapes
        .iter()
        .copied()
        .zip(phase_vector_values(tapes.len(), phase))
        .map(|(tape, value)| format!("{}={}", tape_name(tape), value))
        .collect::<Vec<_>>()
        .join(",")
}

fn format_move_sequence(puzzle: &Puzzle, path: PathBits) -> String {
    let moves = path.to_vec();
    if moves.is_empty() {
        return "identity".to_string();
    }
    moves
        .iter()
        .map(|&move_index| {
            let mv = &puzzle.moves[move_index];
            let sign = if mv.direction > 0 { "+" } else { "-" };
            format!("{}{}", mv.tape_id, sign)
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn print_phase_audit_conflict(
    puzzle: &Puzzle,
    tapes: &[TapeCoord],
    modulus: u8,
    label: &str,
    conflict: PhaseAuditConflict,
) {
    println!(
        "  {label}_phase_conflict: FOUND key={} mod={}",
        conflict.key_kind, modulus
    );
    println!(
        "    existing_phase: [{}]",
        format_phase_vector(tapes, conflict.existing_phase)
    );
    println!(
        "    existing_path: {}",
        format_move_sequence(puzzle, conflict.existing_path)
    );
    println!(
        "    new_phase:      [{}]",
        format_phase_vector(tapes, conflict.new_phase)
    );
    println!(
        "    new_path:      {}",
        format_move_sequence(puzzle, conflict.new_path)
    );
}

fn sticker_axis_family(face: Face) -> u8 {
    match face {
        Face::Left | Face::Right => 0,
        Face::Top | Face::Bottom => 1,
        Face::Front | Face::Back => 2,
    }
}

fn sticker_parity(sticker: &Sticker) -> usize {
    (sticker.x + sticker.y + sticker.z) & 1
}

fn macro_shift_preserves(puzzle: &Puzzle, move_index: MoveIndex, shift: usize) -> (bool, bool) {
    let mv = &puzzle.moves[move_index];
    let len = mv.cycle.len();
    let mut axis_ok = true;
    let mut parity_ok = true;
    for index in 0..len {
        let from = mv.cycle[index];
        let to_position = if mv.direction > 0 {
            (index + shift) % len
        } else {
            (index + len - (shift % len)) % len
        };
        let to = mv.cycle[to_position];
        let from_sticker = &puzzle.stickers[from];
        let to_sticker = &puzzle.stickers[to];
        axis_ok &= sticker_axis_family(from_sticker.face) == sticker_axis_family(to_sticker.face);
        parity_ok &= sticker_parity(from_sticker) == sticker_parity(to_sticker);
    }
    (axis_ok, parity_ok)
}

fn print_macro_preservation_audit(puzzle: &Puzzle) {
    for shift in [1_usize, 2, 3, 6] {
        let mut all_axis_ok = true;
        let mut all_parity_ok = true;
        for move_index in 0..puzzle.moves.len() {
            let (axis_ok, parity_ok) = macro_shift_preserves(puzzle, move_index, shift);
            all_axis_ok &= axis_ok;
            all_parity_ok &= parity_ok;
        }
        println!(
            "macro_shift={} preserves_axis_family={} preserves_parity={}",
            shift, all_axis_ok, all_parity_ok
        );
    }
}

fn optimize_solution(
    puzzle: &Puzzle,
    start_colors: &[Color],
    moves: &[MoveIndex],
) -> Vec<MoveIndex> {
    let mut out = moves.to_vec();
    for _ in 0..10 {
        let before = out.clone();
        out = cancel_adjacent_inverses(puzzle, &out);
        out = compress_same_tape_runs(puzzle, &out);
        out = strip_repeated_state_segments(puzzle, start_colors, &out);
        if out == before {
            break;
        }
    }
    out
}

fn optimize_solution_with_table(
    puzzle: &Puzzle,
    start_colors: &[Color],
    moves: &[MoveIndex],
    table: &HashMap<Key, PathBits>,
    local_window: usize,
    local_depth: usize,
) -> Vec<MoveIndex> {
    let mut out = optimize_solution(puzzle, start_colors, moves);
    if local_window > 1 && local_depth > 0 {
        out = local_rewrite_solution(puzzle, start_colors, &out, local_window, local_depth);
        out = optimize_solution(puzzle, start_colors, &out);
    }
    for _ in 0..6 {
        let before = out.clone();
        out = shortcut_suffix_with_table(puzzle, start_colors, &out, table);
        if local_window > 1 && local_depth > 0 {
            out = local_rewrite_solution(puzzle, start_colors, &out, local_window, local_depth);
        }
        out = optimize_solution(puzzle, start_colors, &out);
        if out == before {
            break;
        }
    }
    if should_try_e_classic_strong_local(puzzle, local_window, local_depth, out.len()) {
        let current = out.clone();
        let mut stronger = local_rewrite_solution(
            puzzle,
            start_colors,
            &out,
            E_CLASSIC_STRONG_LOCAL_WINDOW,
            E_CLASSIC_STRONG_LOCAL_DEPTH,
        );
        stronger = optimize_solution(puzzle, start_colors, &stronger);
        for _ in 0..4 {
            let before = stronger.clone();
            stronger = shortcut_suffix_with_table(puzzle, start_colors, &stronger, table);
            stronger = local_rewrite_solution(
                puzzle,
                start_colors,
                &stronger,
                E_CLASSIC_STRONG_LOCAL_WINDOW,
                E_CLASSIC_STRONG_LOCAL_DEPTH,
            );
            stronger = optimize_solution(puzzle, start_colors, &stronger);
            if stronger == before {
                break;
            }
        }
        if stronger.len() < current.len() {
            out = stronger;
        }
    }
    out
}

fn should_try_e_classic_strong_local(
    puzzle: &Puzzle,
    local_window: usize,
    local_depth: usize,
    solution_len: usize,
) -> bool {
    puzzle.layout == LayoutId::E
        && puzzle.difficulty == Difficulty::Classic
        && solution_len >= E_CLASSIC_STRONG_LOCAL_THRESHOLD
        && (local_window < E_CLASSIC_STRONG_LOCAL_WINDOW
            || local_depth < E_CLASSIC_STRONG_LOCAL_DEPTH)
}

fn shortcut_suffix_with_table(
    puzzle: &Puzzle,
    start_colors: &[Color],
    moves: &[MoveIndex],
    table: &HashMap<Key, PathBits>,
) -> Vec<MoveIndex> {
    let mut best = moves.to_vec();
    let mut colors = start_colors.to_vec();

    if let Some(suffix) = table.get(&state_key(&colors)) {
        let candidate = suffix.to_vec();
        if candidate.len() < best.len() {
            best = candidate;
        }
    }

    for prefix_len in 0..moves.len() {
        puzzle.apply_move(&mut colors, moves[prefix_len]);
        if let Some(suffix) = table.get(&state_key(&colors)) {
            let mut candidate = moves[..=prefix_len].to_vec();
            candidate.extend(suffix.to_vec());
            if candidate.len() < best.len() {
                best = candidate;
            }
        }
    }

    best
}

fn local_rewrite_solution(
    puzzle: &Puzzle,
    start_colors: &[Color],
    moves: &[MoveIndex],
    max_window: usize,
    max_depth: usize,
) -> Vec<MoveIndex> {
    let mut out = moves.to_vec();
    let max_window = max_window.min(out.len());
    if max_window < 2 || max_depth == 0 {
        return out;
    }

    for _pass in 0..6 {
        let prefix_states = build_prefix_states(puzzle, start_colors, &out);
        let mut changed = false;

        'search: for window_len in (2..=max_window).rev() {
            let allowed_depth = max_depth.min(window_len - 1);
            if allowed_depth == 0 {
                continue;
            }

            for start in 0..=out.len() - window_len {
                let end = start + window_len;
                if let Some(shortcut) = shortest_path_between_states(
                    puzzle,
                    &prefix_states[start],
                    &prefix_states[end],
                    allowed_depth,
                ) {
                    if shortcut.len() < window_len {
                        out.splice(start..end, shortcut);
                        changed = true;
                        break 'search;
                    }
                }
            }
        }

        if !changed {
            break;
        }
    }

    out
}

fn build_prefix_states(
    puzzle: &Puzzle,
    start_colors: &[Color],
    moves: &[MoveIndex],
) -> Vec<Vec<Color>> {
    let mut colors = start_colors.to_vec();
    let mut states = Vec::with_capacity(moves.len() + 1);
    states.push(colors.clone());
    for &move_index in moves {
        puzzle.apply_move(&mut colors, move_index);
        states.push(colors.clone());
    }
    states
}

fn shortest_path_between_states(
    puzzle: &Puzzle,
    start_colors: &[Color],
    goal_colors: &[Color],
    max_depth: usize,
) -> Option<Vec<MoveIndex>> {
    if start_colors == goal_colors {
        return Some(Vec::new());
    }

    const LOCAL_STATE_LIMIT: usize = 250_000;
    let reverse_depth = max_depth / 2;
    let forward_depth = max_depth - reverse_depth;
    let table = build_local_reverse_table(puzzle, goal_colors, reverse_depth, LOCAL_STATE_LIMIT)?;
    let start_key = state_key(start_colors);
    if let Some(suffix) = table.get(&start_key) {
        let path = suffix.to_vec();
        if path.len() <= max_depth {
            return Some(path);
        }
    }

    let mut best_hit: Option<Vec<MoveIndex>> = None;
    let mut seen = HashSet::new();
    seen.insert(start_key);
    let mut frontier = vec![ExactEntry {
        colors: start_colors.to_vec(),
        path: PathBits::default(),
        last_move: None,
    }];

    for _ in 0..forward_depth {
        let mut next = Vec::new();
        for entry in frontier {
            for move_index in 0..puzzle.moves.len() {
                if is_immediate_inverse(puzzle, entry.last_move, move_index) {
                    continue;
                }

                let mut colors = entry.colors.clone();
                puzzle.apply_move(&mut colors, move_index);
                let key = state_key(&colors);
                if seen.contains(&key) {
                    continue;
                }

                let path = entry.path.append(move_index);
                if let Some(suffix) = table.get(&key) {
                    if path.len as usize + suffix.len as usize <= max_depth {
                        let mut moves = path.to_vec();
                        moves.extend(suffix.to_vec());
                        keep_shortest(&mut best_hit, moves);
                    }
                    continue;
                }

                seen.insert(key);
                if seen.len() > LOCAL_STATE_LIMIT {
                    return best_hit;
                }
                next.push(ExactEntry {
                    colors,
                    path,
                    last_move: Some(move_index),
                });
            }
        }

        if best_hit.is_some() {
            return best_hit;
        }

        frontier = next;
        if frontier.is_empty() {
            break;
        }
    }

    best_hit
}

fn build_local_reverse_table(
    puzzle: &Puzzle,
    goal_colors: &[Color],
    depth: usize,
    state_limit: usize,
) -> Option<HashMap<Key, PathBits>> {
    let mut states: HashMap<Key, PathBits> = HashMap::new();
    states.insert(state_key(goal_colors), PathBits::default());
    let mut frontier = vec![TableEntry {
        colors: goal_colors.to_vec(),
        path: PathBits::default(),
        last_move: None,
    }];

    for _ in 0..depth {
        let mut next = Vec::new();
        for entry in frontier {
            for move_index in 0..puzzle.moves.len() {
                if is_immediate_inverse(puzzle, entry.last_move, move_index) {
                    continue;
                }

                let mut colors = entry.colors.clone();
                puzzle.apply_move(&mut colors, move_index);
                let key = state_key(&colors);
                if states.contains_key(&key) {
                    continue;
                }

                let inverse = puzzle.inverse_index(move_index);
                let path = entry.path.prepend(inverse);
                states.insert(key, path);
                if states.len() > state_limit {
                    return None;
                }
                next.push(TableEntry {
                    colors,
                    path,
                    last_move: Some(move_index),
                });
            }
        }
        frontier = next;
        if frontier.is_empty() {
            break;
        }
    }

    Some(states)
}

fn cancel_adjacent_inverses(puzzle: &Puzzle, moves: &[MoveIndex]) -> Vec<MoveIndex> {
    let mut stack = Vec::with_capacity(moves.len());
    for &move_index in moves {
        if stack
            .last()
            .is_some_and(|&last| puzzle.inverse_index(last) == move_index)
        {
            stack.pop();
        } else {
            stack.push(move_index);
        }
    }
    stack
}

fn compress_same_tape_runs(puzzle: &Puzzle, moves: &[MoveIndex]) -> Vec<MoveIndex> {
    let mut out = Vec::with_capacity(moves.len());
    let mut i = 0usize;
    while i < moves.len() {
        let mv = &puzzle.moves[moves[i]];
        let axis = mv.axis;
        let layer = mv.layer;
        let cycle_len = mv.cycle.len() as isize;
        let mut net = 0isize;
        let mut j = i;
        while j < moves.len() {
            let current = &puzzle.moves[moves[j]];
            if current.axis != axis || current.layer != layer {
                break;
            }
            net += current.direction as isize;
            j += 1;
        }

        let k = positive_mod(net, cycle_len);
        if k != 0 {
            let pos_steps = k;
            let neg_steps = cycle_len - k;
            let (direction, reps) = if pos_steps <= neg_steps {
                (1, pos_steps as usize)
            } else {
                (-1, neg_steps as usize)
            };
            if let Some(use_index) = puzzle.find_move(axis, layer, direction) {
                out.extend(std::iter::repeat(use_index).take(reps));
            } else {
                out.extend_from_slice(&moves[i..j]);
            }
        }

        i = j;
    }
    out
}

fn strip_repeated_state_segments(
    puzzle: &Puzzle,
    start_colors: &[Color],
    moves: &[MoveIndex],
) -> Vec<MoveIndex> {
    if moves.is_empty() {
        return Vec::new();
    }

    let mut out = moves.to_vec();
    loop {
        let mut colors = start_colors.to_vec();
        let mut seen: HashMap<Vec<Color>, usize> = HashMap::new();
        seen.insert(colors.clone(), 0);
        let mut removed = false;

        for pos in 0..out.len() {
            puzzle.apply_move(&mut colors, out[pos]);
            let prefix_pos = pos + 1;
            if let Some(&prev_pos) = seen.get(&colors) {
                out.drain(prev_pos..prefix_pos);
                removed = true;
                break;
            }
            seen.insert(colors.clone(), prefix_pos);
        }

        if !removed {
            break;
        }
    }
    out
}

fn positive_mod(value: isize, modulus: isize) -> isize {
    ((value % modulus) + modulus) % modulus
}

fn score_state(
    colors: &[Color],
    puzzle: &Puzzle,
    target_mode: TargetMode,
    region_pair_weight: i32,
    pattern_db: Option<&PatternDb>,
) -> i32 {
    let mut disorder = 0_i32;
    let mut unique_penalty = 0_i32;
    let mut solved_faces = 0_i32;
    let mut tail_concentration = 0_i32;
    let mut worst_face = 0_i32;
    let mut pair_bonus = 0_i32;

    for face_indexes in &puzzle.face_indexes {
        let mut counts = [0_i32; 6];
        for &index in face_indexes {
            counts[colors[index] as usize] += 1;
        }

        let mut best = 0_i32;
        let mut unique = 0_i32;
        for &count in &counts {
            if count > 0 {
                unique += 1;
                best = best.max(count);
                pair_bonus += count * count;
            }
        }

        let miss = face_indexes.len() as i32 - best;
        worst_face = worst_face.max(miss);
        disorder += miss;
        unique_penalty += (unique - 1).max(0);
        if miss == 0 {
            solved_faces += 1;
        }

        for &count in &counts {
            if count > 0 && count != best {
                tail_concentration += count * count;
            }
        }
    }

    let mut score = disorder * 1200 + worst_face * 250 + unique_penalty * 250
        - solved_faces * 1200
        - tail_concentration * 10
        - pair_bonus * 2;

    if matches!(target_mode, TargetMode::PairRegion) {
        score += pair_region_score(colors, puzzle);
    } else if !matches!(target_mode, TargetMode::Uniform) {
        score += android_region_pair_score(colors, puzzle, region_pair_weight);
        score += android_partial_pair_score(colors, puzzle);
        score += android_pair_score(colors, puzzle);
    }

    if let Some(pattern_db) = pattern_db {
        score += pattern_db.score_state(colors, puzzle);
    }

    score
}

fn pair_region_score(colors: &[Color], puzzle: &Puzzle) -> i32 {
    let mut disorder = 0_i32;
    let mut unique_penalty = 0_i32;
    let mut solved_regions = 0_i32;
    let mut dominant_classes = Vec::new();

    for (left, right) in opposite_face_pairs() {
        let mut counts = [0_i32; 3];
        for &index in &puzzle.face_indexes[left.index()] {
            counts[color_pair_class(puzzle.difficulty, colors[index]) as usize] += 1;
        }
        for &index in &puzzle.face_indexes[right.index()] {
            counts[color_pair_class(puzzle.difficulty, colors[index]) as usize] += 1;
        }

        let mut best_class = 0usize;
        let mut best_count = 0_i32;
        let mut unique = 0_i32;
        for (class, &count) in counts.iter().enumerate() {
            if count > 0 {
                unique += 1;
                if count > best_count {
                    best_count = count;
                    best_class = class;
                }
            }
        }

        let region_len = (puzzle.face_indexes[left.index()].len()
            + puzzle.face_indexes[right.index()].len()) as i32;
        let miss = region_len - best_count;
        disorder += miss;
        unique_penalty += (unique - 1).max(0);
        if miss == 0 {
            solved_regions += 1;
        }
        dominant_classes.push(best_class);
    }

    dominant_classes.sort_unstable();
    let duplicate_dominants = dominant_classes
        .windows(2)
        .filter(|window| window[0] == window[1])
        .count() as i32;

    disorder * 1400 + unique_penalty * 350 + duplicate_dominants * 900 - solved_regions * 1400
}

fn face_histogram_key(colors: &[Color], puzzle: &Puzzle) -> PatternKey {
    let mut key = [0_u8; FACE_COUNT * 6];
    for (face_index, face_indexes) in puzzle.face_indexes.iter().enumerate() {
        for &index in face_indexes {
            key[face_index * 6 + colors[index] as usize] += 1;
        }
    }
    key
}

fn canonical_face_histogram_key(colors: &[Color], puzzle: &Puzzle) -> PatternKey {
    let mut rows = [[0_u8; 6]; FACE_COUNT];
    for (face_index, face_indexes) in puzzle.face_indexes.iter().enumerate() {
        for &index in face_indexes {
            rows[face_index][colors[index] as usize] += 1;
        }
    }
    rows.sort_unstable();

    let mut key = [0_u8; FACE_COUNT * 6];
    for (face_index, row) in rows.iter().enumerate() {
        for (color, &count) in row.iter().enumerate() {
            key[face_index * 6 + color] = count;
        }
    }
    key
}

fn android_region_pair_score(colors: &[Color], puzzle: &Puzzle, weight: i32) -> i32 {
    if weight == 0 {
        return 0;
    }

    let required = puzzle.difficulty.required_pairs();
    let opposite_faces = [
        (Face::Front, Face::Back),
        (Face::Left, Face::Right),
        (Face::Top, Face::Bottom),
    ];

    let mut costs = [[0_i32; 3]; 3];
    for (region_index, (left, right)) in opposite_faces.iter().copied().enumerate() {
        let possible = (puzzle.face_indexes[left.index()].len()
            + puzzle.face_indexes[right.index()].len()) as i32;
        for (pair_index, &(a, b)) in required.iter().enumerate() {
            let mut matches_pair = 0_i32;
            for &index in &puzzle.face_indexes[left.index()] {
                if colors[index] == a || colors[index] == b {
                    matches_pair += 1;
                }
            }
            for &index in &puzzle.face_indexes[right.index()] {
                if colors[index] == a || colors[index] == b {
                    matches_pair += 1;
                }
            }
            costs[region_index][pair_index] = possible - matches_pair;
        }
    }

    let assignments = [
        [0_usize, 1_usize, 2_usize],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    let best_missing = assignments
        .iter()
        .map(|assignment| {
            assignment
                .iter()
                .enumerate()
                .map(|(region_index, &pair_index)| costs[region_index][pair_index])
                .sum::<i32>()
        })
        .min()
        .unwrap_or(0);

    best_missing * weight
}

fn android_partial_pair_score(colors: &[Color], puzzle: &Puzzle) -> i32 {
    let weight_percent = match puzzle.layout {
        LayoutId::F => 0,
        _ => 100,
    };
    if weight_percent == 0 {
        return 0;
    }

    if uniform_face_colors(colors, &puzzle.face_indexes).is_some() {
        return 0;
    }

    let required = puzzle.difficulty.required_pairs();
    let opposite_faces = [
        (Face::Front, Face::Back),
        (Face::Left, Face::Right),
        (Face::Top, Face::Bottom),
    ];
    let mut score = 0;

    for (left, right) in opposite_faces {
        let (left_color, left_count) =
            dominant_face_color(colors, &puzzle.face_indexes[left.index()]);
        let (right_color, right_count) =
            dominant_face_color(colors, &puzzle.face_indexes[right.index()]);
        let possible =
            puzzle.face_indexes[left.index()].len() + puzzle.face_indexes[right.index()].len();
        let confidence = left_count + right_count;
        if confidence * 2 < possible {
            continue;
        }

        let pair = sorted_pair(left_color, right_color);
        if required.contains(&pair) {
            score -= confidence as i32 * 85;
        } else {
            score += confidence as i32 * 115;
        }
    }

    score * weight_percent / 100
}

fn dominant_face_color(colors: &[Color], indexes: &[usize]) -> (Color, usize) {
    let mut counts = [0_usize; 6];
    for &index in indexes {
        counts[colors[index] as usize] += 1;
    }

    let mut best_color = WHITE;
    let mut best_count = 0usize;
    for (color, &count) in counts.iter().enumerate() {
        if count > best_count {
            best_color = color as Color;
            best_count = count;
        }
    }

    (best_color, best_count)
}

fn android_pair_score(colors: &[Color], puzzle: &Puzzle) -> i32 {
    let Some(uniform) = uniform_face_colors(colors, &puzzle.face_indexes) else {
        return 0;
    };
    let required = puzzle.difficulty.required_pairs();
    let mut actual = actual_pairs_from_uniform(&uniform);
    actual.sort_unstable();

    if actual == required {
        return -4000;
    }

    let mut score = 0;
    for pair in actual {
        if required.contains(&pair) {
            score -= 700;
        } else {
            score += 1800;
        }
    }
    score
}

fn normalize_ariadne_direction(direction: i32, cycle: i32) -> i32 {
    if cycle <= 0 {
        return direction;
    }
    let mut steps = direction.rem_euclid(cycle);
    if steps == 0 {
        return 0;
    }
    if steps as f32 > cycle as f32 / 2.0 {
        steps -= cycle;
    }
    steps
}

fn ariadne_solution_len(puzzle: &Puzzle, history: &[MoveIndex]) -> usize {
    ariadne_reduced_plan(puzzle, history).len()
}

fn ariadne_reduced_plan(puzzle: &Puzzle, history: &[MoveIndex]) -> Vec<AriadneStep> {
    let mut stack: Vec<AriadneStep> = Vec::new();

    for &move_index in history {
        let mv = &puzzle.moves[move_index];
        let cycle = mv.cycle.len() as i32;
        let direction = normalize_ariadne_direction(mv.direction as i32, cycle);
        if direction == 0 {
            continue;
        }

        let mut insert_pos = stack.len();
        while insert_pos > 0 {
            let previous = stack[insert_pos - 1];
            if previous.axis != mv.axis || previous.layer == mv.layer {
                break;
            }
            insert_pos -= 1;
        }

        if insert_pos > 0 {
            let previous = stack[insert_pos - 1];
            if previous.axis == mv.axis && previous.layer == mv.layer {
                let combined = normalize_ariadne_direction(previous.direction + direction, cycle);
                if combined == 0 {
                    stack.remove(insert_pos - 1);
                } else {
                    stack[insert_pos - 1] = AriadneStep {
                        axis: mv.axis,
                        layer: mv.layer,
                        direction: combined,
                    };
                }
                continue;
            }
        }

        stack.insert(
            insert_pos,
            AriadneStep {
                axis: mv.axis,
                layer: mv.layer,
                direction,
            },
        );
    }

    stack
}

fn ariadne_solution_moves(puzzle: &Puzzle, history: &[MoveIndex]) -> Vec<MoveIndex> {
    let plan = ariadne_reduced_plan(puzzle, history);
    let mut out = Vec::new();

    for step in plan.into_iter().rev() {
        let inverse_direction = -step.direction;
        let unit_direction = if inverse_direction > 0 { 1 } else { -1 };
        let count = inverse_direction.unsigned_abs() as usize;
        let move_index = puzzle
            .find_move(step.axis, step.layer, unit_direction)
            .expect("Ariadne step must map to a legal unit move");
        out.extend(std::iter::repeat_n(move_index, count));
    }

    out
}

fn ariadne_plan_text(puzzle: &Puzzle, plan: &[AriadneStep]) -> String {
    plan.iter()
        .map(|step| {
            let move_index = puzzle
                .find_move(
                    step.axis,
                    step.layer,
                    if step.direction > 0 { 1 } else { -1 },
                )
                .expect("Ariadne step must map to a legal unit move");
            let tape_id = &puzzle.moves[move_index].tape_id;
            let sign = if step.direction > 0 { "+" } else { "-" };
            let magnitude = step.direction.unsigned_abs();
            if magnitude == 1 {
                format!("{tape_id}{sign}")
            } else {
                format!("{tape_id}{sign}{magnitude}")
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn generate_scramble(
    puzzle: &Puzzle,
    length: usize,
    seed: u64,
    profile: ScrambleProfile,
    avoid_same_tape: bool,
) -> Vec<MoveIndex> {
    match profile {
        ScrambleProfile::Lab => random_scramble(puzzle, length, seed, avoid_same_tape),
        ScrambleProfile::AndroidOriginal => android_original_scramble(puzzle, length, seed),
    }
}

fn android_original_scramble(puzzle: &Puzzle, length: usize, seed: u64) -> Vec<MoveIndex> {
    let mut rng = Lcg64::new(seed);
    let dims = puzzle.layout.dims();
    let mut last_scramble = Vec::new();

    for _retry in 0..4 {
        let mut scramble = Vec::with_capacity(length);
        let mut last_axis = None;

        for _ in 0..length {
            let axis_pool = [0_u8, 1, 2]
                .into_iter()
                .filter(|axis| Some(*axis) != last_axis)
                .collect::<Vec<_>>();
            let mut selected = None;

            for _attempt in 0..40 {
                let axis = axis_pool[rng.next_usize(axis_pool.len())];
                let layer_count = match axis {
                    0 => dims.rows,
                    1 => dims.cols,
                    _ => dims.layers,
                };
                let layer = rng.next_usize(layer_count);
                if puzzle.difficulty != Difficulty::Easy
                    || !tape_is_uniform(puzzle, axis, layer, &puzzle.solved_colors)
                {
                    selected = Some((axis, layer));
                    break;
                }
            }

            let (axis, layer) = selected.unwrap_or_else(|| {
                let axis = axis_pool[rng.next_usize(axis_pool.len())];
                let layer_count = match axis {
                    0 => dims.rows,
                    1 => dims.cols,
                    _ => dims.layers,
                };
                (axis, rng.next_usize(layer_count))
            });
            let direction = if rng.next_usize(2) == 0 { 1 } else { -1 };
            scramble.push(
                puzzle
                    .find_move(axis, layer, direction)
                    .expect("Android scramble move must exist"),
            );
            last_axis = Some(axis);
        }

        let mut colors = puzzle.solved_colors.clone();
        puzzle.apply_moves(&mut colors, &scramble);
        last_scramble = scramble;
        if !is_android_solved(&colors, &puzzle.face_indexes, puzzle.difficulty) {
            break;
        }
    }

    last_scramble
}

fn tape_is_uniform(puzzle: &Puzzle, axis: u8, layer: usize, colors: &[Color]) -> bool {
    let Some(move_index) = puzzle.find_move(axis, layer, 1) else {
        return false;
    };
    let cycle = &puzzle.moves[move_index].cycle;
    cycle
        .first()
        .is_some_and(|first| cycle.iter().all(|index| colors[*index] == colors[*first]))
}

fn random_scramble(
    puzzle: &Puzzle,
    length: usize,
    seed: u64,
    avoid_same_tape: bool,
) -> Vec<MoveIndex> {
    let mut rng = Lcg64::new(seed);
    let mut out = Vec::with_capacity(length);
    for _ in 0..length {
        let mut selected = None;
        for _attempt in 0..100 {
            let candidate = rng.next_usize(puzzle.moves.len());
            let ok_inverse = out
                .last()
                .is_none_or(|&last| puzzle.inverse_index(last) != candidate);
            let ok_same_tape = !avoid_same_tape
                || out.last().is_none_or(|&last| {
                    let a = &puzzle.moves[last];
                    let b = &puzzle.moves[candidate];
                    a.axis != b.axis || a.layer != b.layer
                });
            if ok_inverse && ok_same_tape {
                selected = Some(candidate);
                break;
            }
        }
        out.push(selected.unwrap_or_else(|| rng.next_usize(puzzle.moves.len())));
    }
    out
}

fn build_stickers(layout: LayoutId) -> Vec<Sticker> {
    let dims = layout.dims();
    let mut stickers = Vec::new();
    for x in 0..dims.rows {
        for y in 0..dims.cols {
            for z in 0..dims.layers {
                for face in FACES {
                    if is_visible(dims, face, x, y, z) {
                        stickers.push(Sticker { face, x, y, z });
                    }
                }
            }
        }
    }
    stickers
}

fn build_sticker_index(stickers: &[Sticker]) -> HashMap<String, usize> {
    stickers
        .iter()
        .enumerate()
        .map(|(index, sticker)| {
            (
                make_key(sticker.face, sticker.x, sticker.y, sticker.z),
                index,
            )
        })
        .collect()
}

fn build_face_indexes(stickers: &[Sticker]) -> [Vec<usize>; FACE_COUNT] {
    let mut out: [Vec<usize>; FACE_COUNT] = std::array::from_fn(|_| Vec::new());
    for (index, sticker) in stickers.iter().enumerate() {
        out[sticker.face.index()].push(index);
    }
    out
}

fn build_tapes(layout: LayoutId, sticker_index_by_id: &HashMap<String, usize>) -> Vec<Tape> {
    let dims = layout.dims();
    let max_x = dims.rows - 1;
    let max_y = dims.cols - 1;
    let max_z = dims.layers - 1;
    let mut tapes = Vec::new();

    for y in 0..=max_y {
        let mut ids = Vec::new();
        for x in 0..=max_x {
            ids.push(make_key(Face::Front, x, y, max_z));
        }
        for z in (0..=max_z).rev() {
            ids.push(make_key(Face::Right, max_x, y, z));
        }
        for x in (0..=max_x).rev() {
            ids.push(make_key(Face::Back, x, y, 0));
        }
        for z in 0..=max_z {
            ids.push(make_key(Face::Left, 0, y, z));
        }
        tapes.push(build_tape(format!("y{y}"), 1, y, ids, sticker_index_by_id));
    }

    for x in 0..=max_x {
        let mut ids = Vec::new();
        for y in (0..=max_y).rev() {
            ids.push(make_key(Face::Front, x, y, max_z));
        }
        for z in (0..=max_z).rev() {
            ids.push(make_key(Face::Bottom, x, 0, z));
        }
        for y in 0..=max_y {
            ids.push(make_key(Face::Back, x, y, 0));
        }
        for z in 0..=max_z {
            ids.push(make_key(Face::Top, x, max_y, z));
        }
        tapes.push(build_tape(format!("x{x}"), 0, x, ids, sticker_index_by_id));
    }

    for z in 0..=max_z {
        let mut ids = Vec::new();
        for x in 0..=max_x {
            ids.push(make_key(Face::Top, x, max_y, z));
        }
        for y in (0..=max_y).rev() {
            ids.push(make_key(Face::Right, max_x, y, z));
        }
        for x in (0..=max_x).rev() {
            ids.push(make_key(Face::Bottom, x, 0, z));
        }
        for y in 0..=max_y {
            ids.push(make_key(Face::Left, 0, y, z));
        }
        tapes.push(build_tape(format!("z{z}"), 2, z, ids, sticker_index_by_id));
    }

    tapes
}

#[derive(Debug, Clone)]
struct Tape {
    id: String,
    axis: u8,
    layer: usize,
    cycle: Vec<usize>,
}

fn build_tape(
    id: String,
    axis: u8,
    layer: usize,
    sticker_ids: Vec<String>,
    sticker_index_by_id: &HashMap<String, usize>,
) -> Tape {
    let cycle = sticker_ids
        .iter()
        .map(|sticker_id| {
            *sticker_index_by_id
                .get(sticker_id)
                .unwrap_or_else(|| panic!("tape {id} references missing sticker {sticker_id}"))
        })
        .collect();

    Tape {
        id,
        axis,
        layer,
        cycle,
    }
}

fn build_moves(tapes: Vec<Tape>) -> Vec<Move> {
    let mut moves = Vec::new();
    for tape in tapes {
        let plus_index = moves.len();
        let minus_index = plus_index + 1;
        moves.push(Move {
            tape_id: tape.id.clone(),
            cycle: tape.cycle.clone(),
            direction: 1,
            axis: tape.axis,
            layer: tape.layer,
            inverse: minus_index,
        });
        moves.push(Move {
            tape_id: tape.id,
            cycle: tape.cycle,
            direction: -1,
            axis: tape.axis,
            layer: tape.layer,
            inverse: plus_index,
        });
    }
    moves
}

fn make_key(face: Face, x: usize, y: usize, z: usize) -> String {
    format!("{}_{}_{}_{}", face.name(), x, y, z)
}

fn is_visible(dims: Dims, face: Face, x: usize, y: usize, z: usize) -> bool {
    let max_x = dims.rows - 1;
    let max_y = dims.cols - 1;
    let max_z = dims.layers - 1;
    match face {
        Face::Front => z == max_z,
        Face::Back => z == 0,
        Face::Left => x == 0,
        Face::Right => x == max_x,
        Face::Top => y == max_y,
        Face::Bottom => y == 0,
    }
}

fn apply_cycle(colors: &mut [Color], cycle: &[usize], direction: i8) {
    if cycle.is_empty() {
        return;
    }

    if direction > 0 {
        let saved = colors[*cycle.last().unwrap()];
        for index in (1..cycle.len()).rev() {
            colors[cycle[index]] = colors[cycle[index - 1]];
        }
        colors[cycle[0]] = saved;
    } else {
        let saved = colors[cycle[0]];
        for index in 0..cycle.len() - 1 {
            colors[cycle[index]] = colors[cycle[index + 1]];
        }
        colors[*cycle.last().unwrap()] = saved;
    }
}

fn is_immediate_inverse(
    puzzle: &Puzzle,
    previous: Option<MoveIndex>,
    move_index: MoveIndex,
) -> bool {
    previous.is_some_and(|previous| puzzle.moves[move_index].inverse == previous)
}

fn is_target_solved(colors: &[Color], puzzle: &Puzzle, target_mode: TargetMode) -> bool {
    match target_mode {
        TargetMode::Uniform => is_uniform_solved(colors, &puzzle.face_indexes),
        TargetMode::PairRegion => is_pair_region_solved(colors, puzzle),
        TargetMode::Android | TargetMode::AndroidMultiGoal | TargetMode::AndroidPortfolio => {
            is_android_solved(colors, &puzzle.face_indexes, puzzle.difficulty)
        }
    }
}

fn is_uniform_solved(colors: &[Color], face_indexes: &[Vec<usize>; FACE_COUNT]) -> bool {
    uniform_face_colors(colors, face_indexes).is_some()
}

fn uniform_face_colors(
    colors: &[Color],
    face_indexes: &[Vec<usize>; FACE_COUNT],
) -> Option<[Color; FACE_COUNT]> {
    let mut out = [0; FACE_COUNT];
    for (face_index, indexes) in face_indexes.iter().enumerate() {
        if indexes.is_empty() {
            return None;
        }
        let first = colors[indexes[0]];
        if indexes.iter().any(|&index| colors[index] != first) {
            return None;
        }
        out[face_index] = first;
    }
    Some(out)
}

fn is_android_solved(
    colors: &[Color],
    face_indexes: &[Vec<usize>; FACE_COUNT],
    difficulty: Difficulty,
) -> bool {
    let Some(uniform) = uniform_face_colors(colors, face_indexes) else {
        return false;
    };
    let mut actual = actual_pairs_from_uniform(&uniform);
    actual.sort_unstable();
    actual == difficulty.required_pairs()
}

fn is_pair_region_solved(colors: &[Color], puzzle: &Puzzle) -> bool {
    let mut region_classes = Vec::new();
    for (left, right) in opposite_face_pairs() {
        let indexes = puzzle.face_indexes[left.index()]
            .iter()
            .chain(puzzle.face_indexes[right.index()].iter())
            .copied()
            .collect::<Vec<_>>();
        if indexes.is_empty() {
            return false;
        }
        let class = color_pair_class(puzzle.difficulty, colors[indexes[0]]);
        if indexes
            .iter()
            .any(|&index| color_pair_class(puzzle.difficulty, colors[index]) != class)
        {
            return false;
        }
        region_classes.push(class);
    }

    region_classes.sort_unstable();
    region_classes == [0, 1, 2]
}

fn actual_pairs_from_uniform(uniform: &[Color; FACE_COUNT]) -> Vec<(Color, Color)> {
    sorted_pairs(&[
        (uniform[Face::Front.index()], uniform[Face::Back.index()]),
        (uniform[Face::Left.index()], uniform[Face::Right.index()]),
        (uniform[Face::Top.index()], uniform[Face::Bottom.index()]),
    ])
}

fn opposite_face_pairs() -> [(Face, Face); 3] {
    [
        (Face::Front, Face::Back),
        (Face::Left, Face::Right),
        (Face::Top, Face::Bottom),
    ]
}

fn sorted_pairs(pairs: &[(Color, Color)]) -> Vec<(Color, Color)> {
    let mut out = pairs
        .iter()
        .map(|&(a, b)| sorted_pair(a, b))
        .collect::<Vec<_>>();
    out.sort_unstable();
    out
}

fn sorted_pair(a: Color, b: Color) -> (Color, Color) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

fn count_colors(colors: &[Color]) -> [usize; 6] {
    let mut counts = [0usize; 6];
    for &color in colors {
        counts[color as usize] += 1;
    }
    counts
}

fn pair_region_colors(puzzle: &Puzzle, colors: &[Color]) -> Vec<Color> {
    colors
        .iter()
        .map(|&color| color_pair_class(puzzle.difficulty, color))
        .collect()
}

fn color_pair_class(difficulty: Difficulty, color: Color) -> Color {
    match difficulty {
        Difficulty::Classic => match color {
            WHITE | YELLOW => 0,
            RED | MAGENTA => 1,
            BLUE | GREEN => 2,
            _ => 0,
        },
        Difficulty::Moderate => match color {
            WHITE => 0,
            RED => 1,
            GREEN => 2,
            _ => 0,
        },
        Difficulty::Easy => match color {
            RED => 0,
            WHITE => 1,
            _ => 0,
        },
    }
}

fn generate_android_solved_states(puzzle: &Puzzle) -> Vec<Vec<Color>> {
    let mut out = Vec::new();
    let mut assignment = [0; FACE_COUNT];
    generate_android_solved_states_inner(puzzle, 0, &mut assignment, &mut out);
    out
}

fn generate_pair_region_solved_states(puzzle: &Puzzle) -> Vec<Vec<Color>> {
    let mut out = Vec::new();
    let assignments = [
        [0_u8, 1_u8, 2_u8],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];

    for assignment in assignments {
        let mut colors = vec![0; puzzle.stickers.len()];
        for (region_index, (left, right)) in opposite_face_pairs().into_iter().enumerate() {
            let class = assignment[region_index];
            for &index in &puzzle.face_indexes[left.index()] {
                colors[index] = class;
            }
            for &index in &puzzle.face_indexes[right.index()] {
                colors[index] = class;
            }
        }
        out.push(colors);
    }

    out
}

fn generate_android_solved_states_inner(
    puzzle: &Puzzle,
    face_pos: usize,
    assignment: &mut [Color; FACE_COUNT],
    out: &mut Vec<Vec<Color>>,
) {
    if face_pos == FACE_COUNT {
        let mut colors = vec![0; puzzle.stickers.len()];
        for face in FACES {
            for &index in &puzzle.face_indexes[face.index()] {
                colors[index] = assignment[face.index()];
            }
        }
        if count_colors(&colors) == puzzle.target_color_counts
            && is_android_solved(&colors, &puzzle.face_indexes, puzzle.difficulty)
        {
            out.push(colors);
        }
        return;
    }

    for color in TARGET_COLORS {
        assignment[face_pos] = color;
        generate_android_solved_states_inner(puzzle, face_pos + 1, assignment, out);
    }
}

fn state_key(colors: &[Color]) -> Key {
    let mut key = [0_u16; 9];
    for (group, slot) in key.iter_mut().enumerate() {
        let mut value = 0_u16;
        let mut multiplier = 1_u16;
        for offset in 0..6 {
            let index = group * 6 + offset;
            if index >= colors.len() {
                break;
            }
            value += colors[index] as u16 * multiplier;
            multiplier *= 6;
        }
        *slot = value;
    }
    key
}

fn hash_colors(colors: &[Color]) -> u32 {
    let mut hash = 2_166_136_261_u32;
    for color in colors {
        hash ^= *color as u32 + 31;
        hash = hash.wrapping_mul(16_777_619);
    }
    hash
}

#[derive(Debug, Clone)]
struct Mulberry32 {
    value: u32,
}

impl Mulberry32 {
    fn new(seed: u32) -> Self {
        Self { value: seed }
    }

    fn next(&mut self) -> u32 {
        self.value = self.value.wrapping_add(0x6d2b_79f5);
        let mut next = self.value;
        next = (next ^ (next >> 15)).wrapping_mul(next | 1);
        next ^= next.wrapping_add((next ^ (next >> 7)).wrapping_mul(next | 61));
        next ^ (next >> 14)
    }
}

fn jitter_sample(rng: &mut Mulberry32, jitter: i32) -> i32 {
    if jitter == 0 {
        return 0;
    }
    let span = (jitter * 2 + 1) as u32;
    rng.next() as i32 % span as i32 - jitter
}

#[derive(Debug, Clone)]
struct Lcg64 {
    state: u64,
}

impl Lcg64 {
    fn new(seed: u64) -> Self {
        Self {
            state: seed ^ 0x9e37_79b9_7f4a_7c15,
        }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state
    }

    fn next_usize(&mut self, upper: usize) -> usize {
        if upper == 0 {
            return 0;
        }
        (self.next_u64() as usize) % upper
    }
}

fn derive_seed(
    base: u64,
    layout: LayoutId,
    difficulty: Difficulty,
    scramble_len: usize,
    iteration: usize,
) -> u64 {
    let mut value = base;
    value ^= (layout as u64 + 1).wrapping_mul(0x9e37_79b9);
    value ^= (difficulty as u64 + 1).wrapping_mul(0x85eb_ca6b);
    value ^= (scramble_len as u64).wrapping_mul(0xc2b2_ae35);
    value ^= (iteration as u64 + 1).wrapping_mul(0x27d4_eb2f);
    value
}

fn unix_stamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn write_csv(path: &PathBuf, records: &[RunRecord]) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(
        file,
        "layout,difficulty,scramble_len,iteration,seed,ariadne_solution_len,found,reason,method,target_used,operation_profile_used,raw_solution_len,optimized_solution_len,optimized_changed,uniform_solved,android_solved,nodes,elapsed_ms,first_table_hit,first_table_hit_target,first_table_hit_profile,first_table_hit_rank,first_table_hit_pattern_db,first_table_hit_depth,first_table_hit_restart,first_table_hit_nodes,first_table_hit_elapsed_ms,first_table_hit_prefix_len,first_table_hit_suffix_len,first_table_hit_total_len,gain_vs_scramble,ratio_vs_scramble,scramble_script,raw_solution_script,optimized_solution_script"
    )?;
    for r in records {
        writeln!(
            file,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{:.4},{},{},{}",
            r.layout,
            r.difficulty,
            r.scramble_len,
            r.iteration,
            r.seed,
            r.ariadne_solution_len,
            r.found,
            csv(&r.reason),
            r.method.label(),
            r.target_used,
            r.operation_profile_used,
            r.raw_solution_len,
            r.optimized_solution_len,
            r.optimized_changed,
            r.uniform_solved,
            r.android_solved,
            r.nodes,
            r.elapsed_ms,
            r.first_table_hit,
            r.first_table_hit_target,
            r.first_table_hit_profile,
            r.first_table_hit_rank,
            r.first_table_hit_pattern_db,
            r.first_table_hit_depth,
            r.first_table_hit_restart,
            r.first_table_hit_nodes,
            r.first_table_hit_elapsed_ms,
            r.first_table_hit_prefix_len,
            r.first_table_hit_suffix_len,
            r.first_table_hit_total_len,
            r.gain_vs_scramble,
            r.ratio_vs_scramble,
            csv(&r.scramble_script),
            csv(&r.raw_solution_script),
            csv(&r.optimized_solution_script)
        )?;
    }
    Ok(())
}

fn write_ariadne_check_csv(path: &PathBuf, records: &[AriadneCheckRecord]) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(
        file,
        "layout,difficulty,scramble_len,iteration,seed,ariadne_stack_len,ariadne_unit_len,exact_solved,uniform_solved,android_solved,scramble_script,ariadne_stack_script,ariadne_solution_script"
    )?;
    for r in records {
        writeln!(
            file,
            "{},{},{},{},{},{},{},{},{},{},{},{},{}",
            r.layout,
            r.difficulty,
            r.scramble_len,
            r.iteration,
            r.seed,
            r.ariadne_stack_len,
            r.ariadne_unit_len,
            r.exact_solved,
            r.uniform_solved,
            r.android_solved,
            csv(&r.scramble_script),
            csv(&r.ariadne_stack_script),
            csv(&r.ariadne_solution_script)
        )?;
    }
    Ok(())
}

fn write_heuristic_audit_csv(path: &PathBuf, records: &[HeuristicAuditRecord]) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(
        file,
        "layout,difficulty,target,depth,states,histogram_keys,canonical_histogram_keys,histogram_ratio,canonical_histogram_ratio,nodes,elapsed_ms,pearson_score_distance,truncated,reason,depth_counts,mean_score_by_depth"
    )?;
    for r in records {
        writeln!(
            file,
            "{},{},{},{},{},{},{},{:.6},{:.6},{},{},{:.6},{},{},{},{}",
            r.layout,
            r.difficulty,
            r.target.label(),
            r.depth,
            r.states,
            r.histogram_keys,
            r.canonical_histogram_keys,
            ratio(r.histogram_keys, r.states),
            ratio(r.canonical_histogram_keys, r.states),
            r.nodes,
            r.elapsed_ms,
            r.pearson_score_distance,
            r.truncated,
            csv(&r.reason),
            csv(&join_usize(&r.depth_counts)),
            csv(&join_f64(&r.mean_score_by_depth))
        )?;
    }
    Ok(())
}

fn write_projection_audit_csv(path: &PathBuf, records: &[ProjectionAuditRecord]) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(
        file,
        "layout,difficulty,target,projection,depth,states,keys,key_ratio,mean_depth_span,p95_depth_span,max_depth_span,mean_states_per_key,p95_states_per_key,max_states_per_key,nodes,elapsed_ms,truncated,reason"
    )?;
    for r in records {
        writeln!(
            file,
            "{},{},{},{},{},{},{},{:.6},{:.6},{},{},{:.6},{},{},{},{},{},{}",
            r.layout,
            r.difficulty,
            r.target.label(),
            r.projection.label(),
            r.depth,
            r.states,
            r.keys,
            ratio(r.keys, r.states),
            r.mean_depth_span,
            r.p95_depth_span,
            r.max_depth_span,
            r.mean_states_per_key,
            r.p95_states_per_key,
            r.max_states_per_key,
            r.nodes,
            r.elapsed_ms,
            r.truncated,
            csv(&r.reason)
        )?;
    }
    Ok(())
}

fn write_axis_ring_pdb_audit_csv(path: &PathBuf, records: &[AxisRingPdbRecord]) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(
        file,
        "layout,difficulty,target,seed_source,scramble_len,iteration,seed,axis_table_depth,target_expand_depth,pdb_depth,target_states,pdb_states,depth_counts,build_nodes,build_elapsed_ms,truncated,reason,ariadne_unit_len,path_states,hit_count,start_hit_distance,first_hit_step,first_hit_distance,first_hit_remaining_to_solved,best_hit_step,best_hit_distance,best_hit_remaining_to_solved,best_prefix_to_seed,max_axis_suffix,estimated_total_with_max_suffix,path_distance_remaining_pearson"
    )?;
    for r in records {
        writeln!(
            file,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{:.6}",
            r.layout,
            r.difficulty,
            r.target.label(),
            r.seed_source.label(),
            r.scramble_len,
            r.iteration,
            r.seed,
            r.axis_table_depth,
            r.target_expand_depth,
            r.pdb_depth,
            r.target_states,
            r.pdb_states,
            csv(&join_usize(&r.depth_counts)),
            r.build_nodes,
            r.build_elapsed_ms,
            r.truncated,
            csv(&r.reason),
            r.ariadne_unit_len,
            r.path_states,
            r.hit_count,
            fmt_opt_u8(r.start_hit_distance),
            fmt_opt_usize(r.first_hit_step),
            fmt_opt_u8(r.first_hit_distance),
            fmt_opt_usize(r.first_hit_remaining_to_solved),
            fmt_opt_usize(r.best_hit_step),
            fmt_opt_u8(r.best_hit_distance),
            fmt_opt_usize(r.best_hit_remaining_to_solved),
            fmt_opt_usize(r.best_prefix_to_axis_ring),
            r.max_axis_suffix,
            fmt_opt_usize(r.estimated_total_with_max_suffix),
            r.path_distance_remaining_pearson
        )?;
    }
    Ok(())
}

fn write_macro_two_stage_csv(path: &PathBuf, records: &[MacroTwoStageRecord]) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(
        file,
        "layout,difficulty,scramble_len,iteration,seed,ariadne_solution_len,found,reason,prefix_len,suffix_macro_len,suffix_unit_len,raw_solution_len,optimized_solution_len,android_solved,nodes,elapsed_ms,scramble_script,raw_solution_script,optimized_solution_script"
    )?;
    for r in records {
        writeln!(
            file,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            r.layout,
            r.difficulty,
            r.scramble_len,
            r.iteration,
            r.seed,
            r.ariadne_solution_len,
            r.found,
            csv(&r.reason),
            r.prefix_len,
            r.suffix_macro_len,
            r.suffix_unit_len,
            r.raw_solution_len,
            r.optimized_solution_len,
            r.android_solved,
            r.nodes,
            r.elapsed_ms,
            csv(&r.scramble_script),
            csv(&r.raw_solution_script),
            csv(&r.optimized_solution_script)
        )?;
    }
    Ok(())
}

fn write_exact_shortcut_csv(path: &PathBuf, records: &[ExactShortcutRecord]) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(
        file,
        "layout,difficulty,target,scramble_len,iteration,seed,ariadne_solution_len,table_depth,forward_depth,proof_bound,table_states,table_nodes,table_elapsed_ms,found,proved_optimal,optimal_len,shortcut_moves,shortcut_ratio,significant_shortcut,search_nodes,search_elapsed_ms,forward_seen,table_hits,complete,reason,scramble_script,optimal_script"
    )?;
    for r in records {
        writeln!(
            file,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{:.6},{},{},{},{},{},{},{},{},{}",
            r.layout,
            r.difficulty,
            r.target_mode.label(),
            r.scramble_len,
            r.iteration,
            r.seed,
            r.ariadne_solution_len,
            r.table_depth,
            r.forward_depth,
            r.proof_bound,
            r.table_states,
            r.table_nodes,
            r.table_elapsed_ms,
            r.found,
            r.proved_optimal,
            r.optimal_len,
            r.shortcut_moves,
            r.shortcut_ratio,
            r.significant_shortcut,
            r.search_nodes,
            r.search_elapsed_ms,
            r.forward_seen,
            r.table_hits,
            r.complete,
            csv(&r.reason),
            csv(&r.scramble_script),
            csv(&r.optimal_script)
        )?;
    }
    Ok(())
}

fn write_exact_shortcut_markdown(
    path: &PathBuf,
    options: &BenchOptions,
    records: &[ExactShortcutRecord],
) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(file, "# SkimmIQ Exact Shortcut Audit")?;
    writeln!(file)?;
    writeln!(file, "- target: `{}`", options.solver.target_mode.label())?;
    writeln!(
        file,
        "- scramble_profile: `{}`",
        options.scramble_profile.label()
    )?;
    writeln!(file, "- layouts: `{}`", join_display(&options.layouts))?;
    writeln!(
        file,
        "- difficulties: `{}`",
        join_display(&options.difficulties)
    )?;
    writeln!(file, "- scrambles: `{:?}`", options.scramble_lengths)?;
    writeln!(file, "- iterations: `{}`", iteration_label(options))?;
    writeln!(file, "- seed: `{}`", options.seed)?;
    if let Some(first) = records.first() {
        writeln!(file, "- table_depth: `{}`", first.table_depth)?;
        writeln!(file, "- forward_depth: `{}`", first.forward_depth)?;
        writeln!(file, "- proof_bound: `{}`", first.proof_bound)?;
    }
    writeln!(file)?;
    writeln!(
        file,
        "This audit asks whether SkimmIQ has real color-class shortcuts compared with Ariadne. The goal is `android-multi`: any valid Android solved color assignment, not a fixed sticker permutation. `Proved optimal` is only true when the bounded meet-in-the-middle search completed; otherwise a found path is only a bounded witness."
    )?;
    writeln!(file)?;

    let mut groups: BTreeMap<(LayoutId, Difficulty, String, usize), Vec<&ExactShortcutRecord>> =
        BTreeMap::new();
    for record in records {
        groups
            .entry((
                record.layout,
                record.difficulty,
                record.target_mode.label().to_string(),
                record.scramble_len,
            ))
            .or_default()
            .push(record);
    }

    writeln!(
        file,
        "| Layout | Difficulty | Target | Scramble | Samples | Found | Proved | Significant Shortcuts | Mean Ariadne | Mean Opt | Mean Shortcut | Mean Ratio | P95 Opt | Mean Search ms | Mean Nodes | Mean Seen | Mean Hits | Reasons |"
    )?;
    writeln!(
        file,
        "|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|"
    )?;
    for ((layout, difficulty, target, scramble_len), group) in &groups {
        let found = group.iter().filter(|record| record.found).count();
        let proved = group.iter().filter(|record| record.proved_optimal).count();
        let significant = group
            .iter()
            .filter(|record| record.significant_shortcut)
            .count();
        let ariadne = group
            .iter()
            .map(|record| record.ariadne_solution_len)
            .collect::<Vec<_>>();
        let opt = group
            .iter()
            .filter(|record| record.found)
            .map(|record| record.optimal_len)
            .collect::<Vec<_>>();
        let shortcuts = group
            .iter()
            .filter(|record| record.found)
            .map(|record| record.shortcut_moves)
            .collect::<Vec<_>>();
        let ratios = group
            .iter()
            .filter(|record| record.found)
            .map(|record| record.shortcut_ratio)
            .collect::<Vec<_>>();
        let times = group
            .iter()
            .map(|record| record.search_elapsed_ms)
            .collect::<Vec<_>>();
        let nodes = group
            .iter()
            .map(|record| record.search_nodes)
            .collect::<Vec<_>>();
        let seen = group
            .iter()
            .map(|record| record.forward_seen)
            .collect::<Vec<_>>();
        let hits = group
            .iter()
            .map(|record| record.table_hits)
            .collect::<Vec<_>>();
        let mut reasons = BTreeMap::new();
        for record in group {
            *reasons.entry(record.reason.clone()).or_insert(0) += 1;
        }
        writeln!(
            file,
            "| {} | {} | `{}` | {} | {} | {}/{} | {}/{} | {}/{} | {} | {} | {} | {} | {} | {} | {} | {} | {} | `{}` |",
            layout,
            difficulty,
            target,
            scramble_len,
            group.len(),
            found,
            group.len(),
            proved,
            group.len(),
            significant,
            group.len(),
            fmt_opt_f64(mean_usize(&ariadne)),
            fmt_opt_f64(mean_usize(&opt)),
            fmt_opt_f64(mean_isize(&shortcuts)),
            fmt_opt_f64(mean_f64(&ratios)),
            fmt_opt_usize(percentile_usize(&opt, 0.95)),
            fmt_opt_f64(mean_u128(&times)),
            fmt_opt_f64(mean_u64(&nodes)),
            fmt_opt_f64(mean_usize(&seen)),
            fmt_opt_f64(mean_usize(&hits)),
            join_counts(&reasons)
        )?;
    }

    writeln!(file)?;
    writeln!(file, "## Iteration Details")?;
    writeln!(file)?;
    writeln!(
        file,
        "| Iter | Scramble | Seed | Ariadne | Found | Proved | Opt | Shortcut | Ratio | Significant | Nodes | Seen | Hits | Time ms | Reason |"
    )?;
    writeln!(
        file,
        "|---:|---:|---:|---:|---|---|---:|---:|---:|---|---:|---:|---:|---:|---|"
    )?;
    for record in records {
        writeln!(
            file,
            "| {} | {} | {} | {} | {} | {} | {} | {} | {:.3} | {} | {} | {} | {} | {} | `{}` |",
            record.iteration,
            record.scramble_len,
            record.seed,
            record.ariadne_solution_len,
            record.found,
            record.proved_optimal,
            record.optimal_len,
            record.shortcut_moves,
            record.shortcut_ratio,
            record.significant_shortcut,
            record.search_nodes,
            record.forward_seen,
            record.table_hits,
            record.search_elapsed_ms,
            record.reason
        )?;
    }

    Ok(())
}

fn write_axis_ring_csv(path: &PathBuf, records: &[AxisRingRecord]) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(
        file,
        "layout,difficulty,scramble_len,iteration,seed,axis,ariadne_solution_len,found,reason,prefix_len,suffix_len,raw_solution_len,optimized_solution_len,android_solved,nodes,elapsed_ms,scramble_script,raw_solution_script,optimized_solution_script"
    )?;
    for r in records {
        writeln!(
            file,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            r.layout,
            r.difficulty,
            r.scramble_len,
            r.iteration,
            r.seed,
            axis_label(r.axis),
            r.ariadne_solution_len,
            r.found,
            csv(&r.reason),
            r.prefix_len,
            r.suffix_len,
            r.raw_solution_len,
            r.optimized_solution_len,
            r.android_solved,
            r.nodes,
            r.elapsed_ms,
            csv(&r.scramble_script),
            csv(&r.raw_solution_script),
            csv(&r.optimized_solution_script)
        )?;
    }
    Ok(())
}

fn write_move_delta_audit_csv(path: &PathBuf, records: &[MoveDeltaAuditRecord]) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(
        file,
        "layout,difficulty,target,operation_profile,scramble_len,iteration,seed,ariadne_unit_len,step,step_bucket,remaining_to_solved,ariadne_move,feature,candidate_count,first_match_count,raw_rank,raw_percentile,raw_delta,first_match_rank,first_match_percentile,first_match_delta,best_delta,worst_delta"
    )?;
    for r in records {
        writeln!(
            file,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{:.3},{:.6},{:.6},{:.3},{:.6},{:.6},{:.6},{:.6}",
            r.layout,
            r.difficulty,
            r.target.label(),
            r.operation_profile.label(),
            r.scramble_len,
            r.iteration,
            r.seed,
            r.ariadne_unit_len,
            r.step,
            r.step_bucket,
            r.remaining_to_solved,
            csv(&r.ariadne_move),
            r.feature,
            r.candidate_count,
            r.first_match_count,
            r.raw_rank,
            r.raw_percentile,
            r.raw_delta,
            r.first_match_rank,
            r.first_match_percentile,
            r.first_match_delta,
            r.best_delta,
            r.worst_delta
        )?;
    }
    Ok(())
}

fn write_feature_cost_csv(path: &PathBuf, records: &[FeatureCostRecord]) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(
        file,
        "layout,difficulty,scramble_len,states,repeats,feature,evals,total_ns,mean_ns,checksum"
    )?;
    for r in records {
        writeln!(
            file,
            "{},{},{},{},{},{},{},{},{:.3},{}",
            r.layout,
            r.difficulty,
            r.scramble_len,
            r.states,
            r.repeats,
            r.feature,
            r.evals,
            r.total_ns,
            r.mean_ns,
            r.checksum
        )?;
    }
    Ok(())
}

fn write_commutator_scan_csv(path: &PathBuf, records: &[CommutatorScanRecord]) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(
        file,
        "layout,difficulty,max_len,sequences,pairs_examined,unique_permutations,identity_count,min_support,clean_3_cycles,double_transpositions,elapsed_ms,truncated,reason,histogram"
    )?;
    for r in records {
        writeln!(
            file,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            r.layout,
            r.difficulty,
            r.max_len,
            r.sequences,
            r.pairs_examined,
            r.unique_permutations,
            r.identity_count,
            r.min_support,
            r.clean_3_cycles,
            r.double_transpositions,
            r.elapsed_ms,
            r.truncated,
            csv(&r.reason),
            csv(&join_counts(&r.histogram))
        )?;
    }
    Ok(())
}

fn write_commutator_catalog_csv(
    path: &PathBuf,
    records: &[CommutatorScanRecord],
) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(
        file,
        "layout,difficulty,max_len,rank,support,cycle_type,cycle_lengths,a,b,full_commutator,moved_positions,cycles,sticker_refs"
    )?;
    for r in records {
        let puzzle = Puzzle::new(r.layout, r.difficulty);
        for (rank, candidate) in r.catalog.iter().enumerate() {
            let permutation = operation_permutation_key(&puzzle, &candidate.moves);
            writeln!(
                file,
                "{},{},{},{},{},{},{},{},{},{},{},{},{}",
                r.layout,
                r.difficulty,
                r.max_len,
                rank + 1,
                candidate.support,
                csv(&candidate.cycle_type),
                csv(&join_usize(&candidate.cycle_lengths)),
                csv(&puzzle.moves_text(&candidate.a)),
                csv(&puzzle.moves_text(&candidate.b)),
                csv(&puzzle.moves_text(&candidate.moves)),
                csv(&moved_positions_text(&permutation)),
                csv(&permutation_cycles_text(&permutation)),
                csv(&sticker_refs_text(&puzzle, &permutation))
            )?;
        }
    }
    Ok(())
}

fn write_commutator_applicability_csv(
    path: &PathBuf,
    records: &[CommutatorApplicabilityRecord],
) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(
        file,
        "layout,difficulty,target,scramble_len,iteration,seed,initial_mismatches,clean_3_catalog_size,distinct_triples,covered_positions,improving_commutators,strong_commutators,best_delta,best_after_mismatches,best_direction,best_commutator"
    )?;
    for r in records {
        writeln!(
            file,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            r.layout,
            r.difficulty,
            r.target_mode.label(),
            r.scramble_len,
            r.iteration,
            r.seed,
            r.initial_mismatches,
            r.clean_3_catalog_size,
            r.distinct_triples,
            r.covered_positions,
            r.improving_commutators,
            r.strong_commutators,
            r.best_delta,
            r.best_after_mismatches,
            csv(&r.best_direction),
            csv(&r.best_commutator)
        )?;
    }
    Ok(())
}

fn write_commutator_applicability_markdown(
    path: &PathBuf,
    options: &BenchOptions,
    records: &[CommutatorApplicabilityRecord],
) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(file, "# SkimmIQ Commutator Applicability Audit")?;
    writeln!(file)?;
    writeln!(file, "- target: `{}`", options.solver.target_mode.label())?;
    writeln!(file, "- layouts: `{}`", join_display(&options.layouts))?;
    writeln!(
        file,
        "- difficulties: `{}`",
        join_display(&options.difficulties)
    )?;
    writeln!(file, "- scrambles: `{:?}`", options.scramble_lengths)?;
    writeln!(file, "- iterations: `{}`", iteration_label(options))?;
    writeln!(file, "- seed: `{}`", options.seed)?;
    writeln!(file, "- max_len(A/B): `{}`", options.commutator_max_len)?;
    writeln!(file)?;
    writeln!(
        file,
        "This audit checks whether the clean 3-cycle commutator catalog has direct color-state leverage. `Improving` counts forward/inverse catalog applications that reduce mismatches to the best initial Android target. `Strong` counts applications improving by at least two stickers."
    )?;
    writeln!(file)?;

    let mut groups: BTreeMap<(LayoutId, Difficulty, usize), Vec<&CommutatorApplicabilityRecord>> =
        BTreeMap::new();
    for record in records {
        groups
            .entry((record.layout, record.difficulty, record.scramble_len))
            .or_default()
            .push(record);
    }

    writeln!(
        file,
        "| Layout | Difficulty | Scramble | Samples | Zero Improving | Mean Improving | P50 Improving | Mean Strong | P50 Strong | Mean Best Delta | P95 Best Delta | Mean Initial Mismatches | Catalog 3-cycles | Distinct Triples | Covered Positions |"
    )?;
    writeln!(
        file,
        "|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|"
    )?;
    for ((layout, difficulty, scramble_len), group) in groups {
        let improving = group
            .iter()
            .map(|record| record.improving_commutators)
            .collect::<Vec<_>>();
        let strong = group
            .iter()
            .map(|record| record.strong_commutators)
            .collect::<Vec<_>>();
        let best_deltas = group
            .iter()
            .map(|record| record.best_delta)
            .collect::<Vec<_>>();
        let best_deltas_as_usize = best_deltas
            .iter()
            .map(|&delta| delta.max(0) as usize)
            .collect::<Vec<_>>();
        let initial = group
            .iter()
            .map(|record| record.initial_mismatches)
            .collect::<Vec<_>>();
        let zero_improving = group
            .iter()
            .filter(|record| record.improving_commutators == 0)
            .count();
        let first = group[0];
        writeln!(
            file,
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            layout,
            difficulty,
            scramble_len,
            group.len(),
            zero_improving,
            fmt_opt_f64(mean_usize(&improving)),
            fmt_opt_usize(percentile_usize(&improving, 0.50)),
            fmt_opt_f64(mean_usize(&strong)),
            fmt_opt_usize(percentile_usize(&strong, 0.50)),
            fmt_opt_f64(mean_isize(&best_deltas)),
            fmt_opt_usize(percentile_usize(&best_deltas_as_usize, 0.95)),
            fmt_opt_f64(mean_usize(&initial)),
            first.clean_3_catalog_size,
            first.distinct_triples,
            first.covered_positions
        )?;
    }

    Ok(())
}

fn write_commutator_greedy_csv(
    path: &PathBuf,
    records: &[CommutatorGreedyRecord],
) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(
        file,
        "layout,difficulty,target,scramble_len,iteration,seed,ariadne_solution_len,initial_mismatches,final_mismatches,found,reason,commutator_steps,greedy_raw_len,greedy_optimized_len,suffix_attempted,suffix_found,suffix_raw_len,suffix_optimized_len,suffix_reason,raw_solution_len,optimized_solution_len,best_step_delta,mean_step_delta,elapsed_ms,raw_solution_script,optimized_solution_script"
    )?;
    for r in records {
        writeln!(
            file,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{:.3},{},{},{}",
            r.layout,
            r.difficulty,
            r.target_mode.label(),
            r.scramble_len,
            r.iteration,
            r.seed,
            r.ariadne_solution_len,
            r.initial_mismatches,
            r.final_mismatches,
            r.found,
            csv(&r.reason),
            r.commutator_steps,
            r.greedy_raw_len,
            r.greedy_optimized_len,
            r.suffix_attempted,
            r.suffix_found,
            r.suffix_raw_len,
            r.suffix_optimized_len,
            csv(&r.suffix_reason),
            r.raw_solution_len,
            r.optimized_solution_len,
            r.best_step_delta,
            r.mean_step_delta,
            r.elapsed_ms,
            csv(&r.raw_solution_script),
            csv(&r.optimized_solution_script)
        )?;
    }
    Ok(())
}

fn write_commutator_greedy_markdown(
    path: &PathBuf,
    options: &BenchOptions,
    records: &[CommutatorGreedyRecord],
) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(file, "# SkimmIQ Commutator Greedy Audit")?;
    writeln!(file)?;
    writeln!(file, "- target: `{}`", options.solver.target_mode.label())?;
    writeln!(file, "- layouts: `{}`", join_display(&options.layouts))?;
    writeln!(
        file,
        "- difficulties: `{}`",
        join_display(&options.difficulties)
    )?;
    writeln!(file, "- scrambles: `{:?}`", options.scramble_lengths)?;
    writeln!(file, "- iterations: `{}`", iteration_label(options))?;
    writeln!(file, "- seed: `{}`", options.seed)?;
    writeln!(file, "- max_len(A/B): `{}`", options.commutator_max_len)?;
    writeln!(
        file,
        "- greedy_steps: `{}`",
        options.commutator_greedy_steps
    )?;
    writeln!(
        file,
        "- dynamic_target: `{}`",
        options.commutator_dynamic_target
    )?;
    writeln!(
        file,
        "- plateau_lookahead: `{}`",
        options.commutator_plateau_lookahead
    )?;
    writeln!(
        file,
        "- suffix_rescue: `{}`",
        options.commutator_suffix_rescue
    )?;
    writeln!(
        file,
        "- suffix_time_limit_ms: `{}`",
        options.commutator_suffix_time_limit_ms
    )?;
    writeln!(file)?;
    writeln!(
        file,
        "This audit repeatedly applies the catalog commutator primitive that most reduces color mismatches. With `dynamic_target=false` the target is fixed at the nearest initial Android solved state; with `dynamic_target=true` each state is scored against the nearest Android solved state. It is a robustness/fallback probe, not a shortest-path solver."
    )?;
    writeln!(file)?;

    let mut groups: BTreeMap<(LayoutId, Difficulty, usize), Vec<&CommutatorGreedyRecord>> =
        BTreeMap::new();
    for record in records {
        groups
            .entry((record.layout, record.difficulty, record.scramble_len))
            .or_default()
            .push(record);
    }

    writeln!(
        file,
        "| Layout | Difficulty | Scramble | Samples | Found | Suffix Found | Mean Steps | Mean Greedy Opt | Mean Suffix Opt | Mean Final Opt | P95 Final Opt | Mean Final Mismatch | P95 Final Mismatch | Mean Time ms | Reasons |"
    )?;
    writeln!(
        file,
        "|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|"
    )?;
    for ((layout, difficulty, scramble_len), group) in groups {
        let found = group.iter().filter(|record| record.found).count();
        let steps = group
            .iter()
            .map(|record| record.commutator_steps)
            .collect::<Vec<_>>();
        let opt = group
            .iter()
            .map(|record| record.optimized_solution_len)
            .collect::<Vec<_>>();
        let greedy_opt = group
            .iter()
            .map(|record| record.greedy_optimized_len)
            .collect::<Vec<_>>();
        let suffix_opt = group
            .iter()
            .filter(|record| record.suffix_attempted)
            .map(|record| record.suffix_optimized_len)
            .collect::<Vec<_>>();
        let suffix_found = group.iter().filter(|record| record.suffix_found).count();
        let final_mismatches = group
            .iter()
            .map(|record| record.final_mismatches)
            .collect::<Vec<_>>();
        let times = group
            .iter()
            .map(|record| record.elapsed_ms)
            .collect::<Vec<_>>();
        let mut reasons = BTreeMap::new();
        for record in &group {
            *reasons.entry(record.reason.clone()).or_insert(0) += 1;
        }
        writeln!(
            file,
            "| {} | {} | {} | {} | {}/{} | {}/{} | {} | {} | {} | {} | {} | {} | {} | {} | `{}` |",
            layout,
            difficulty,
            scramble_len,
            group.len(),
            found,
            group.len(),
            suffix_found,
            group.len(),
            fmt_opt_f64(mean_usize(&steps)),
            fmt_opt_f64(mean_usize(&greedy_opt)),
            fmt_opt_f64(mean_usize(&suffix_opt)),
            fmt_opt_f64(mean_usize(&opt)),
            fmt_opt_usize(percentile_usize(&opt, 0.95)),
            fmt_opt_f64(mean_usize(&final_mismatches)),
            fmt_opt_usize(percentile_usize(&final_mismatches, 0.95)),
            fmt_opt_f64(mean_u128(&times)),
            join_counts(&reasons)
        )?;
    }

    Ok(())
}

fn write_commutator_plateau_csv(
    path: &PathBuf,
    records: &[CommutatorPlateauRecord],
) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(
        file,
        "layout,difficulty,target,scramble_len,iteration,seed,greedy_reason,greedy_steps,greedy_raw_len,initial_mismatches,plateau_mismatches,best_dynamic_mismatches,residual_canonical_parity,residual_flex_parity,full_canonical_parity,support_touch_1,support_touch_2,support_touch_3,support_touch_4,support_subset_mismatch,support_contains_all_mismatch,direct_improving,direct_nonworsening,direct_best_delta,direct_best_after,mismatch_positions,mismatch_details,transition_counts"
    )?;
    for r in records {
        writeln!(
            file,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            r.layout,
            r.difficulty,
            r.target_mode.label(),
            r.scramble_len,
            r.iteration,
            r.seed,
            csv(&r.greedy_reason),
            r.greedy_steps,
            r.greedy_raw_len,
            r.initial_mismatches,
            r.plateau_mismatches,
            r.best_dynamic_mismatches,
            csv(&r.residual_canonical_parity),
            csv(&r.residual_flex_parity),
            csv(&r.full_canonical_parity),
            r.support_touch_1,
            r.support_touch_2,
            r.support_touch_3,
            r.support_touch_4,
            r.support_subset_mismatch,
            r.support_contains_all_mismatch,
            r.direct_improving,
            r.direct_nonworsening,
            r.direct_best_delta,
            r.direct_best_after,
            csv(&r.mismatch_positions),
            csv(&r.mismatch_details),
            csv(&r.transition_counts)
        )?;
    }
    Ok(())
}

fn write_commutator_plateau_markdown(
    path: &PathBuf,
    options: &BenchOptions,
    records: &[CommutatorPlateauRecord],
) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(file, "# SkimmIQ Commutator Plateau Audit")?;
    writeln!(file)?;
    writeln!(file, "- target: `{}`", options.solver.target_mode.label())?;
    writeln!(file, "- layouts: `{}`", join_display(&options.layouts))?;
    writeln!(
        file,
        "- difficulties: `{}`",
        join_display(&options.difficulties)
    )?;
    writeln!(file, "- scrambles: `{:?}`", options.scramble_lengths)?;
    writeln!(file, "- iterations: `{}`", iteration_label(options))?;
    writeln!(file, "- seed: `{}`", options.seed)?;
    writeln!(file, "- max_len(A/B): `{}`", options.commutator_max_len)?;
    writeln!(
        file,
        "- dynamic_target: `{}`",
        options.commutator_dynamic_target
    )?;
    writeln!(
        file,
        "- plateau_lookahead: `{}`",
        options.commutator_plateau_lookahead
    )?;
    writeln!(file)?;
    writeln!(
        file,
        "This audit inspects the state left by commutator greedy before suffix rescue. `Residual flex parity` is computed only on mismatched positions, allowing same-color ambiguity inside the residue; `both` means parity is not a hard color-level blocker."
    )?;
    writeln!(file)?;

    let mut groups: BTreeMap<(LayoutId, Difficulty, usize), Vec<&CommutatorPlateauRecord>> =
        BTreeMap::new();
    for record in records {
        groups
            .entry((record.layout, record.difficulty, record.scramble_len))
            .or_default()
            .push(record);
    }

    writeln!(
        file,
        "| Layout | Difficulty | Scramble | Samples | Mean Plateau | P95 Plateau | Odd Canonical | Flex Both | Flex Odd Only | Mean Touch1 | Mean Touch2 | Mean Touch3 | Mean Touch4 | Mean Direct Improving | Mean Direct Nonworsening | Mean Best Delta | Reasons |"
    )?;
    writeln!(
        file,
        "|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|"
    )?;
    for ((layout, difficulty, scramble_len), group) in &groups {
        let plateau = group
            .iter()
            .map(|record| record.plateau_mismatches)
            .collect::<Vec<_>>();
        let touch1 = group
            .iter()
            .map(|record| record.support_touch_1)
            .collect::<Vec<_>>();
        let touch2 = group
            .iter()
            .map(|record| record.support_touch_2)
            .collect::<Vec<_>>();
        let touch3 = group
            .iter()
            .map(|record| record.support_touch_3)
            .collect::<Vec<_>>();
        let touch4 = group
            .iter()
            .map(|record| record.support_touch_4)
            .collect::<Vec<_>>();
        let direct_improving = group
            .iter()
            .map(|record| record.direct_improving)
            .collect::<Vec<_>>();
        let direct_nonworsening = group
            .iter()
            .map(|record| record.direct_nonworsening)
            .collect::<Vec<_>>();
        let best_deltas = group
            .iter()
            .map(|record| record.direct_best_delta)
            .collect::<Vec<_>>();
        let odd_canonical = group
            .iter()
            .filter(|record| record.residual_canonical_parity == "odd")
            .count();
        let flex_both = group
            .iter()
            .filter(|record| record.residual_flex_parity == "both")
            .count();
        let flex_odd_only = group
            .iter()
            .filter(|record| record.residual_flex_parity == "odd-only")
            .count();
        let mut reasons = BTreeMap::new();
        for record in group {
            *reasons.entry(record.greedy_reason.clone()).or_insert(0) += 1;
        }
        writeln!(
            file,
            "| {} | {} | {} | {} | {} | {} | {}/{} | {}/{} | {}/{} | {} | {} | {} | {} | {} | {} | {} | `{}` |",
            layout,
            difficulty,
            scramble_len,
            group.len(),
            fmt_opt_f64(mean_usize(&plateau)),
            fmt_opt_usize(percentile_usize(&plateau, 0.95)),
            odd_canonical,
            group.len(),
            flex_both,
            group.len(),
            flex_odd_only,
            group.len(),
            fmt_opt_f64(mean_usize(&touch1)),
            fmt_opt_f64(mean_usize(&touch2)),
            fmt_opt_f64(mean_usize(&touch3)),
            fmt_opt_f64(mean_usize(&touch4)),
            fmt_opt_f64(mean_usize(&direct_improving)),
            fmt_opt_f64(mean_usize(&direct_nonworsening)),
            fmt_opt_f64(mean_isize(&best_deltas)),
            join_counts(&reasons)
        )?;
    }

    writeln!(file)?;
    writeln!(file, "## Plateau Details")?;
    writeln!(file)?;
    writeln!(
        file,
        "| Iteration | Seed | Plateau | Best Dynamic | Residual Parity | Flex Parity | Touch 1/2/3/4 | Direct Best Delta | Direct Improving | Mismatch Positions | Transitions |"
    )?;
    writeln!(file, "|---:|---:|---:|---:|---|---|---|---:|---:|---|---|")?;
    for record in records {
        writeln!(
            file,
            "| {} | {} | {} | {} | `{}` | `{}` | {}/{}/{}/{} | {} | {} | `{}` | `{}` |",
            record.iteration,
            record.seed,
            record.plateau_mismatches,
            record.best_dynamic_mismatches,
            record.residual_canonical_parity,
            record.residual_flex_parity,
            record.support_touch_1,
            record.support_touch_2,
            record.support_touch_3,
            record.support_touch_4,
            record.direct_best_delta,
            record.direct_improving,
            record.mismatch_positions,
            record.transition_counts
        )?;
    }

    Ok(())
}

fn write_commutator_endgame_csv(
    path: &PathBuf,
    records: &[CommutatorEndgameRecord],
) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(
        file,
        "layout,difficulty,target,scramble_len,iteration,seed,ariadne_solution_len,greedy_reason,greedy_steps,greedy_raw_len,plateau_mismatches,found,reason,endgame_found,endgame_depth,endgame_nodes,endgame_raw_len,endgame_best_mismatches,total_raw_len,total_optimized_len,final_mismatches,elapsed_ms,mismatch_positions,endgame_script,optimized_script"
    )?;
    for r in records {
        writeln!(
            file,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            r.layout,
            r.difficulty,
            r.target_mode.label(),
            r.scramble_len,
            r.iteration,
            r.seed,
            r.ariadne_solution_len,
            csv(&r.greedy_reason),
            r.greedy_steps,
            r.greedy_raw_len,
            r.plateau_mismatches,
            r.found,
            csv(&r.reason),
            r.endgame_found,
            r.endgame_depth,
            r.endgame_nodes,
            r.endgame_raw_len,
            r.endgame_best_mismatches,
            r.total_raw_len,
            r.total_optimized_len,
            r.final_mismatches,
            r.elapsed_ms,
            csv(&r.mismatch_positions),
            csv(&r.endgame_script),
            csv(&r.optimized_script)
        )?;
    }
    Ok(())
}

fn write_commutator_endgame_markdown(
    path: &PathBuf,
    options: &BenchOptions,
    records: &[CommutatorEndgameRecord],
) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(file, "# SkimmIQ Commutator Endgame Audit")?;
    writeln!(file)?;
    writeln!(file, "- target: `{}`", options.solver.target_mode.label())?;
    writeln!(file, "- layouts: `{}`", join_display(&options.layouts))?;
    writeln!(
        file,
        "- difficulties: `{}`",
        join_display(&options.difficulties)
    )?;
    writeln!(file, "- scrambles: `{:?}`", options.scramble_lengths)?;
    writeln!(file, "- iterations: `{}`", iteration_label(options))?;
    writeln!(file, "- seed: `{}`", options.seed)?;
    writeln!(file, "- max_len(A/B): `{}`", options.commutator_max_len)?;
    writeln!(
        file,
        "- greedy_steps: `{}`",
        options.commutator_greedy_steps
    )?;
    writeln!(
        file,
        "- dynamic_target: `{}`",
        options.commutator_dynamic_target
    )?;
    writeln!(
        file,
        "- plateau_lookahead: `{}`",
        options.commutator_plateau_lookahead
    )?;
    writeln!(
        file,
        "- endgame_depth: `{}`",
        options.commutator_endgame_depth
    )?;
    writeln!(
        file,
        "- endgame_width: `{}`",
        options.commutator_endgame_width
    )?;
    writeln!(
        file,
        "- endgame_time_limit_ms: `{}`",
        options.commutator_endgame_time_limit_ms
    )?;
    writeln!(file)?;
    writeln!(
        file,
        "This audit runs the commutator greedy reducer first, then uses a bounded beam over clean 3-cycles and double transpositions to repair the small residue. Success is judged primarily as robustness/coverage of the algebraic fallback, not as beating the current ring-portfolio mean."
    )?;
    writeln!(file)?;

    let mut groups: BTreeMap<(LayoutId, Difficulty, usize), Vec<&CommutatorEndgameRecord>> =
        BTreeMap::new();
    for record in records {
        groups
            .entry((record.layout, record.difficulty, record.scramble_len))
            .or_default()
            .push(record);
    }

    writeln!(
        file,
        "| Layout | Difficulty | Scramble | Samples | Found | Endgame Found | Mean Ariadne | Mean Plateau | P95 Plateau | Mean Endgame Depth | Mean Endgame Raw | Mean Endgame Best Mismatch | Mean Final Opt | P95 Final Opt | Mean Final Mismatch | Mean Nodes | Mean Time ms | Reasons |"
    )?;
    writeln!(
        file,
        "|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|"
    )?;
    for ((layout, difficulty, scramble_len), group) in &groups {
        let found = group.iter().filter(|record| record.found).count();
        let endgame_found = group.iter().filter(|record| record.endgame_found).count();
        let ariadne = group
            .iter()
            .map(|record| record.ariadne_solution_len)
            .collect::<Vec<_>>();
        let plateau = group
            .iter()
            .map(|record| record.plateau_mismatches)
            .collect::<Vec<_>>();
        let endgame_depth = group
            .iter()
            .map(|record| record.endgame_depth)
            .collect::<Vec<_>>();
        let endgame_raw = group
            .iter()
            .map(|record| record.endgame_raw_len)
            .collect::<Vec<_>>();
        let endgame_best_mismatches = group
            .iter()
            .map(|record| record.endgame_best_mismatches)
            .collect::<Vec<_>>();
        let final_opt = group
            .iter()
            .map(|record| record.total_optimized_len)
            .collect::<Vec<_>>();
        let final_mismatches = group
            .iter()
            .map(|record| record.final_mismatches)
            .collect::<Vec<_>>();
        let nodes = group
            .iter()
            .map(|record| record.endgame_nodes)
            .collect::<Vec<_>>();
        let times = group
            .iter()
            .map(|record| record.elapsed_ms)
            .collect::<Vec<_>>();
        let mut reasons = BTreeMap::new();
        for record in group {
            *reasons.entry(record.reason.clone()).or_insert(0) += 1;
        }
        writeln!(
            file,
            "| {} | {} | {} | {} | {}/{} | {}/{} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | `{}` |",
            layout,
            difficulty,
            scramble_len,
            group.len(),
            found,
            group.len(),
            endgame_found,
            group.len(),
            fmt_opt_f64(mean_usize(&ariadne)),
            fmt_opt_f64(mean_usize(&plateau)),
            fmt_opt_usize(percentile_usize(&plateau, 0.95)),
            fmt_opt_f64(mean_usize(&endgame_depth)),
            fmt_opt_f64(mean_usize(&endgame_raw)),
            fmt_opt_f64(mean_usize(&endgame_best_mismatches)),
            fmt_opt_f64(mean_usize(&final_opt)),
            fmt_opt_usize(percentile_usize(&final_opt, 0.95)),
            fmt_opt_f64(mean_usize(&final_mismatches)),
            fmt_opt_f64(mean_u64(&nodes)),
            fmt_opt_f64(mean_u128(&times)),
            join_counts(&reasons)
        )?;
    }

    writeln!(file)?;
    writeln!(file, "## Iteration Details")?;
    writeln!(file)?;
    writeln!(
        file,
        "| Iteration | Seed | Ariadne | Plateau | Found | Reason | Endgame Found | Endgame Depth | Endgame Nodes | Endgame Raw | Endgame Best Mismatch | Total Raw | Total Opt | Final Mismatch | Mismatch Positions |"
    )?;
    writeln!(
        file,
        "|---:|---:|---:|---:|---|---|---|---:|---:|---:|---:|---:|---:|---:|---|"
    )?;
    for record in records {
        writeln!(
            file,
            "| {} | {} | {} | {} | {} | `{}` | {} | {} | {} | {} | {} | {} | {} | {} | `{}` |",
            record.iteration,
            record.seed,
            record.ariadne_solution_len,
            record.plateau_mismatches,
            record.found,
            record.reason,
            record.endgame_found,
            record.endgame_depth,
            record.endgame_nodes,
            record.endgame_raw_len,
            record.endgame_best_mismatches,
            record.total_raw_len,
            record.total_optimized_len,
            record.final_mismatches,
            record.mismatch_positions
        )?;
    }

    Ok(())
}

fn write_commutator_decomposition_csv(
    path: &PathBuf,
    records: &[CommutatorDecompositionRecord],
) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(
        file,
        "layout,difficulty,target,scramble_len,iteration,seed,ariadne_solution_len,plateau_mismatches,plateau_positions,exact_support_available,subset_support_count,contains_all_support_count,target_min_mismatches,target_max_mismatches,target_min_count,direct_found,direct_reason,direct_steps,direct_raw_len,direct_opt_len,direct_total_opt_len,direct_final_mismatches,unit_found,unit_reason,unit_raw_len,unit_opt_len,unit_total_opt_len,unit_final_mismatches,unit_nodes,unit_elapsed_ms,helper_found,helper_reason,helper_steps,helper_raw_len,helper_opt_len,helper_total_opt_len,helper_final_mismatches,helper_nodes,helper_elapsed_ms,helper_endgame_found,helper_endgame_reason,helper_endgame_raw_len,helper_endgame_opt_len,helper_endgame_total_opt_len,helper_endgame_final_mismatches,helper_endgame_nodes,helper_endgame_elapsed_ms"
    )?;
    for r in records {
        let row = vec![
            r.layout.to_string(),
            r.difficulty.to_string(),
            r.target_mode.label().to_string(),
            r.scramble_len.to_string(),
            r.iteration.to_string(),
            r.seed.to_string(),
            r.ariadne_solution_len.to_string(),
            r.plateau_mismatches.to_string(),
            csv(&r.plateau_positions),
            r.exact_support_available.to_string(),
            r.subset_support_count.to_string(),
            r.contains_all_support_count.to_string(),
            r.target_min_mismatches.to_string(),
            r.target_max_mismatches.to_string(),
            r.target_min_count.to_string(),
            r.direct_found.to_string(),
            csv(&r.direct_reason),
            r.direct_steps.to_string(),
            r.direct_raw_len.to_string(),
            r.direct_opt_len.to_string(),
            r.direct_total_opt_len.to_string(),
            r.direct_final_mismatches.to_string(),
            r.unit_found.to_string(),
            csv(&r.unit_reason),
            r.unit_raw_len.to_string(),
            r.unit_opt_len.to_string(),
            r.unit_total_opt_len.to_string(),
            r.unit_final_mismatches.to_string(),
            r.unit_nodes.to_string(),
            r.unit_elapsed_ms.to_string(),
            r.helper_found.to_string(),
            csv(&r.helper_reason),
            r.helper_steps.to_string(),
            r.helper_raw_len.to_string(),
            r.helper_opt_len.to_string(),
            r.helper_total_opt_len.to_string(),
            r.helper_final_mismatches.to_string(),
            r.helper_nodes.to_string(),
            r.helper_elapsed_ms.to_string(),
            r.helper_endgame_found.to_string(),
            csv(&r.helper_endgame_reason),
            r.helper_endgame_raw_len.to_string(),
            r.helper_endgame_opt_len.to_string(),
            r.helper_endgame_total_opt_len.to_string(),
            r.helper_endgame_final_mismatches.to_string(),
            r.helper_endgame_nodes.to_string(),
            r.helper_endgame_elapsed_ms.to_string(),
        ];
        writeln!(file, "{}", row.join(","))?;
    }
    Ok(())
}

fn write_commutator_decomposition_markdown(
    path: &PathBuf,
    options: &BenchOptions,
    records: &[CommutatorDecompositionRecord],
) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(file, "# SkimmIQ Commutator Decomposition Audit")?;
    writeln!(file)?;
    writeln!(file, "- target: `{}`", options.solver.target_mode.label())?;
    writeln!(file, "- layouts: `{}`", join_display(&options.layouts))?;
    writeln!(
        file,
        "- difficulties: `{}`",
        join_display(&options.difficulties)
    )?;
    writeln!(file, "- scrambles: `{:?}`", options.scramble_lengths)?;
    writeln!(file, "- iterations: `{}`", iteration_label(options))?;
    writeln!(file, "- seed: `{}`", options.seed)?;
    writeln!(file, "- max_len(A/B): `{}`", options.commutator_max_len)?;
    writeln!(
        file,
        "- unit_closure_time_limit_ms: `{}`",
        options.commutator_suffix_time_limit_ms
    )?;
    writeln!(file)?;
    writeln!(
        file,
        "This audit compares two ways to close the same greedy plateau residue: a direct deterministic commutator-greedy tail and a unit-move-only beam control. It is the first falsification step before building an algebraic tail or unified ring+tail solver."
    )?;
    writeln!(file)?;

    let mut groups: BTreeMap<(LayoutId, Difficulty, usize), Vec<&CommutatorDecompositionRecord>> =
        BTreeMap::new();
    for record in records {
        groups
            .entry((record.layout, record.difficulty, record.scramble_len))
            .or_default()
            .push(record);
    }

    writeln!(
        file,
        "| Layout | Difficulty | Scramble | Samples | Mean Plateau | Exact Support | Direct Found | Mean Direct Tail Opt | Mean Direct Total Opt | Mean Direct Final Mismatch | Helper Found | Mean Helper Tail Opt | Mean Helper Total Opt | Mean Helper Final Mismatch | Mean Helper Time ms | Helper+Endgame Found | Mean Helper+Endgame Tail Opt | Mean Helper+Endgame Total Opt | Mean Helper+Endgame Final Mismatch | Mean Helper+Endgame Time ms | Mean Helper+Endgame Nodes | Unit Found | Mean Unit Tail Opt | Mean Unit Total Opt | Mean Unit Final Mismatch | Mean Unit Time ms | Mean Unit Nodes |"
    )?;
    writeln!(
        file,
        "|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|"
    )?;
    for ((layout, difficulty, scramble_len), group) in &groups {
        let plateau = group
            .iter()
            .map(|record| record.plateau_mismatches)
            .collect::<Vec<_>>();
        let direct_tail = group
            .iter()
            .map(|record| record.direct_opt_len)
            .collect::<Vec<_>>();
        let direct_total = group
            .iter()
            .map(|record| record.direct_total_opt_len)
            .collect::<Vec<_>>();
        let direct_mismatch = group
            .iter()
            .map(|record| record.direct_final_mismatches)
            .collect::<Vec<_>>();
        let unit_tail = group
            .iter()
            .map(|record| record.unit_opt_len)
            .collect::<Vec<_>>();
        let unit_total = group
            .iter()
            .map(|record| record.unit_total_opt_len)
            .collect::<Vec<_>>();
        let unit_mismatch = group
            .iter()
            .map(|record| record.unit_final_mismatches)
            .collect::<Vec<_>>();
        let unit_times = group
            .iter()
            .map(|record| record.unit_elapsed_ms)
            .collect::<Vec<_>>();
        let unit_nodes = group
            .iter()
            .map(|record| record.unit_nodes)
            .collect::<Vec<_>>();
        let helper_tail = group
            .iter()
            .map(|record| record.helper_opt_len)
            .collect::<Vec<_>>();
        let helper_total = group
            .iter()
            .map(|record| record.helper_total_opt_len)
            .collect::<Vec<_>>();
        let helper_mismatch = group
            .iter()
            .map(|record| record.helper_final_mismatches)
            .collect::<Vec<_>>();
        let helper_times = group
            .iter()
            .map(|record| record.helper_elapsed_ms)
            .collect::<Vec<_>>();
        let helper_endgame_tail = group
            .iter()
            .map(|record| record.helper_endgame_opt_len)
            .collect::<Vec<_>>();
        let helper_endgame_total = group
            .iter()
            .map(|record| record.helper_endgame_total_opt_len)
            .collect::<Vec<_>>();
        let helper_endgame_mismatch = group
            .iter()
            .map(|record| record.helper_endgame_final_mismatches)
            .collect::<Vec<_>>();
        let helper_endgame_times = group
            .iter()
            .map(|record| record.helper_endgame_elapsed_ms)
            .collect::<Vec<_>>();
        let helper_endgame_nodes = group
            .iter()
            .map(|record| record.helper_endgame_nodes)
            .collect::<Vec<_>>();
        let exact_support = group
            .iter()
            .filter(|record| record.exact_support_available)
            .count();
        let direct_found = group.iter().filter(|record| record.direct_found).count();
        let helper_found = group.iter().filter(|record| record.helper_found).count();
        let helper_endgame_found = group
            .iter()
            .filter(|record| record.helper_endgame_found)
            .count();
        let unit_found = group.iter().filter(|record| record.unit_found).count();
        writeln!(
            file,
            "| {} | {} | {} | {} | {} | {}/{} | {}/{} | {} | {} | {} | {}/{} | {} | {} | {} | {} | {}/{} | {} | {} | {} | {} | {} | {}/{} | {} | {} | {} | {} | {} |",
            layout,
            difficulty,
            scramble_len,
            group.len(),
            fmt_opt_f64(mean_usize(&plateau)),
            exact_support,
            group.len(),
            direct_found,
            group.len(),
            fmt_opt_f64(mean_usize(&direct_tail)),
            fmt_opt_f64(mean_usize(&direct_total)),
            fmt_opt_f64(mean_usize(&direct_mismatch)),
            helper_found,
            group.len(),
            fmt_opt_f64(mean_usize(&helper_tail)),
            fmt_opt_f64(mean_usize(&helper_total)),
            fmt_opt_f64(mean_usize(&helper_mismatch)),
            fmt_opt_f64(mean_u128(&helper_times)),
            helper_endgame_found,
            group.len(),
            fmt_opt_f64(mean_usize(&helper_endgame_tail)),
            fmt_opt_f64(mean_usize(&helper_endgame_total)),
            fmt_opt_f64(mean_usize(&helper_endgame_mismatch)),
            fmt_opt_f64(mean_u128(&helper_endgame_times)),
            fmt_opt_f64(mean_u64(&helper_endgame_nodes)),
            unit_found,
            group.len(),
            fmt_opt_f64(mean_usize(&unit_tail)),
            fmt_opt_f64(mean_usize(&unit_total)),
            fmt_opt_f64(mean_usize(&unit_mismatch)),
            fmt_opt_f64(mean_u128(&unit_times)),
            fmt_opt_f64(mean_u64(&unit_nodes))
        )?;
    }

    writeln!(file)?;
    writeln!(file, "## Iteration Details")?;
    writeln!(file)?;
    writeln!(
        file,
        "| Iter | Seed | Ariadne | Plateau | Exact Support | Subset Support | Contains All | Target Min/Max/Count | Direct Found | Direct Tail/Total | Direct Mismatch | Helper Found | Helper Tail/Total | Helper Mismatch | Helper Time ms | Helper Reason | Helper+Endgame Found | Helper+Endgame Tail/Total | Helper+Endgame Mismatch | Helper+Endgame Time ms | Helper+Endgame Reason | Unit Found | Unit Tail/Total | Unit Mismatch | Unit Time ms | Unit Reason |"
    )?;
    writeln!(
        file,
        "|---:|---:|---:|---:|---|---:|---:|---|---|---|---:|---|---|---:|---:|---|---|---|---:|---:|---|---|---|---:|---:|---|"
    )?;
    for record in records {
        writeln!(
            file,
            "| {} | {} | {} | {} | {} | {} | {} | {}/{}/{} | {} | {}/{} | {} | {} | {}/{} | {} | {} | `{}` | {} | {}/{} | {} | {} | `{}` | {} | {}/{} | {} | {} | `{}` |",
            record.iteration,
            record.seed,
            record.ariadne_solution_len,
            record.plateau_mismatches,
            record.exact_support_available,
            record.subset_support_count,
            record.contains_all_support_count,
            record.target_min_mismatches,
            record.target_max_mismatches,
            record.target_min_count,
            record.direct_found,
            record.direct_opt_len,
            record.direct_total_opt_len,
            record.direct_final_mismatches,
            record.helper_found,
            record.helper_opt_len,
            record.helper_total_opt_len,
            record.helper_final_mismatches,
            record.helper_elapsed_ms,
            record.helper_reason,
            record.helper_endgame_found,
            record.helper_endgame_opt_len,
            record.helper_endgame_total_opt_len,
            record.helper_endgame_final_mismatches,
            record.helper_endgame_elapsed_ms,
            record.helper_endgame_reason,
            record.unit_found,
            record.unit_opt_len,
            record.unit_total_opt_len,
            record.unit_final_mismatches,
            record.unit_elapsed_ms,
            record.unit_reason
        )?;
    }

    Ok(())
}

fn write_commutator_branch_csv(
    path: &PathBuf,
    records: &[CommutatorBranchAuditRecord],
) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(
        file,
        "layout,difficulty,target,scramble_len,iteration,seed,source,gate,step,ariadne_remaining,mismatches,filtered_primitives,filtered_directions,capped_primitives,touch_1,touch_2,touch_3,touch_4_or_more"
    )?;
    for r in records {
        writeln!(
            file,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            r.layout,
            r.difficulty,
            r.target_mode.label(),
            r.scramble_len,
            r.iteration,
            r.seed,
            csv(&r.source),
            r.gate,
            r.step,
            r.ariadne_remaining,
            r.mismatches,
            r.filtered_primitives,
            r.filtered_directions,
            r.capped_primitives,
            r.touch_1,
            r.touch_2,
            r.touch_3,
            r.touch_4_or_more
        )?;
    }
    Ok(())
}

fn write_commutator_branch_markdown(
    path: &PathBuf,
    options: &BenchOptions,
    records: &[CommutatorBranchAuditRecord],
) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(file, "# SkimmIQ Commutator Branch Audit")?;
    writeln!(file)?;
    writeln!(file, "- target: `{}`", options.solver.target_mode.label())?;
    writeln!(file, "- layouts: `{}`", join_display(&options.layouts))?;
    writeln!(
        file,
        "- difficulties: `{}`",
        join_display(&options.difficulties)
    )?;
    writeln!(file, "- scrambles: `{:?}`", options.scramble_lengths)?;
    writeln!(file, "- iterations: `{}`", iteration_label(options))?;
    writeln!(file, "- seed: `{}`", options.seed)?;
    writeln!(file, "- max_len(A/B): `{}`", options.commutator_max_len)?;
    writeln!(
        file,
        "- branch_cap_from_commutator_top: `{}`",
        options.commutator_top
    )?;
    writeln!(file)?;
    writeln!(
        file,
        "This audit measures the branch inflation of injecting the clean 3-cycle/double-transposition catalog only when a state is near solved. `Filtered primitives` counts catalog primitives touching at least one currently mismatched position; each primitive has two directions. It does not solve the state."
    )?;
    writeln!(file)?;

    let mut groups: BTreeMap<
        (LayoutId, Difficulty, usize, String, usize),
        Vec<&CommutatorBranchAuditRecord>,
    > = BTreeMap::new();
    for record in records {
        groups
            .entry((
                record.layout,
                record.difficulty,
                record.scramble_len,
                record.source.clone(),
                record.gate,
            ))
            .or_default()
            .push(record);
    }

    writeln!(
        file,
        "| Layout | Difficulty | Scramble | Source | Gate | Samples | Mean Mismatch | Mean Ariadne Remaining | Mean Filtered | P50 Filtered | P95 Filtered | Mean Directions | Mean Capped | Zero Applicable | Mean Touch1 | Mean Touch2 | Mean Touch3 | Mean Touch4+ |"
    )?;
    writeln!(
        file,
        "|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|"
    )?;
    for ((layout, difficulty, scramble_len, source, gate), group) in &groups {
        let mismatches = group
            .iter()
            .map(|record| record.mismatches)
            .collect::<Vec<_>>();
        let remaining = group
            .iter()
            .map(|record| record.ariadne_remaining)
            .collect::<Vec<_>>();
        let filtered = group
            .iter()
            .map(|record| record.filtered_primitives)
            .collect::<Vec<_>>();
        let directions = group
            .iter()
            .map(|record| record.filtered_directions)
            .collect::<Vec<_>>();
        let capped = group
            .iter()
            .map(|record| record.capped_primitives)
            .collect::<Vec<_>>();
        let touch1 = group
            .iter()
            .map(|record| record.touch_1)
            .collect::<Vec<_>>();
        let touch2 = group
            .iter()
            .map(|record| record.touch_2)
            .collect::<Vec<_>>();
        let touch3 = group
            .iter()
            .map(|record| record.touch_3)
            .collect::<Vec<_>>();
        let touch4 = group
            .iter()
            .map(|record| record.touch_4_or_more)
            .collect::<Vec<_>>();
        let zero = group
            .iter()
            .filter(|record| record.filtered_primitives == 0)
            .count();
        writeln!(
            file,
            "| {} | {} | {} | `{}` | {} | {} | {} | {} | {} | {} | {} | {} | {} | {}/{} | {} | {} | {} | {} |",
            layout,
            difficulty,
            scramble_len,
            source,
            gate,
            group.len(),
            fmt_opt_f64(mean_usize(&mismatches)),
            fmt_opt_f64(mean_usize(&remaining)),
            fmt_opt_f64(mean_usize(&filtered)),
            fmt_opt_usize(percentile_usize(&filtered, 0.50)),
            fmt_opt_usize(percentile_usize(&filtered, 0.95)),
            fmt_opt_f64(mean_usize(&directions)),
            fmt_opt_f64(mean_usize(&capped)),
            zero,
            group.len(),
            fmt_opt_f64(mean_usize(&touch1)),
            fmt_opt_f64(mean_usize(&touch2)),
            fmt_opt_f64(mean_usize(&touch3)),
            fmt_opt_f64(mean_usize(&touch4))
        )?;
    }

    Ok(())
}

fn write_ring_residue_csv(path: &PathBuf, records: &[RingResidueAuditRecord]) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(
        file,
        "layout,difficulty,target,scramble_len,iteration,seed,ariadne_solution_len,gate,prefix_found,prefix_reason,prefix_rank,prefix_profile,prefix_len,prefix_mismatches,prefix_nodes,prefix_elapsed_ms,tail_found,tail_reason,tail_raw_len,tail_opt_len,tail_mismatches,tail_nodes,tail_elapsed_ms,total_found,total_opt_len,final_mismatches"
    )?;
    for r in records {
        writeln!(
            file,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            r.layout,
            r.difficulty,
            r.target_mode.label(),
            r.scramble_len,
            r.iteration,
            r.seed,
            r.ariadne_solution_len,
            r.gate,
            r.prefix_found,
            csv(&r.prefix_reason),
            csv(&r.prefix_rank),
            csv(&r.prefix_profile),
            r.prefix_len,
            r.prefix_mismatches,
            r.prefix_nodes,
            r.prefix_elapsed_ms,
            r.tail_found,
            csv(&r.tail_reason),
            r.tail_raw_len,
            r.tail_opt_len,
            r.tail_mismatches,
            r.tail_nodes,
            r.tail_elapsed_ms,
            r.total_found,
            r.total_opt_len,
            r.final_mismatches
        )?;
    }
    Ok(())
}

fn write_ring_residue_markdown(
    path: &PathBuf,
    options: &BenchOptions,
    records: &[RingResidueAuditRecord],
) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(file, "# SkimmIQ Ring Prefix To Residue Audit")?;
    writeln!(file)?;
    writeln!(file, "- target: `{}`", options.solver.target_mode.label())?;
    writeln!(file, "- layouts: `{}`", join_display(&options.layouts))?;
    writeln!(
        file,
        "- difficulties: `{}`",
        join_display(&options.difficulties)
    )?;
    writeln!(file, "- scrambles: `{:?}`", options.scramble_lengths)?;
    writeln!(file, "- iterations: `{}`", iteration_label(options))?;
    writeln!(file, "- seed: `{}`", options.seed)?;
    writeln!(
        file,
        "- endgame_depth: `{}` width=`{}` time_limit_ms=`{}`",
        options.commutator_endgame_depth,
        options.commutator_endgame_width,
        options.commutator_endgame_time_limit_ms
    )?;
    writeln!(file)?;
    writeln!(
        file,
        "This audit stops the ring/beam prefix as soon as it reaches a small color residue, then hands that state to the commutator endgame search. It tests whether early stop can reduce the minute-level cost of full ring-portfolio."
    )?;
    writeln!(file)?;

    let mut groups: BTreeMap<(LayoutId, Difficulty, usize, usize), Vec<&RingResidueAuditRecord>> =
        BTreeMap::new();
    for record in records {
        groups
            .entry((
                record.layout,
                record.difficulty,
                record.scramble_len,
                record.gate,
            ))
            .or_default()
            .push(record);
    }

    writeln!(
        file,
        "| Layout | Difficulty | Scramble | K | Samples | Prefix Found | Tail Found | Total Found | Mean Prefix Len | Mean Prefix Mismatch | Mean Prefix Time ms | Mean Tail Opt | Mean Tail Time ms | Mean Total Opt | P95 Total Opt | Mean Final Mismatch | Reasons |"
    )?;
    writeln!(
        file,
        "|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|"
    )?;
    for ((layout, difficulty, scramble_len, gate), group) in &groups {
        let prefix_found = group.iter().filter(|record| record.prefix_found).count();
        let tail_found = group.iter().filter(|record| record.tail_found).count();
        let total_found = group.iter().filter(|record| record.total_found).count();
        let prefix_len = group
            .iter()
            .map(|record| record.prefix_len)
            .collect::<Vec<_>>();
        let prefix_mismatches = group
            .iter()
            .map(|record| record.prefix_mismatches)
            .collect::<Vec<_>>();
        let prefix_times = group
            .iter()
            .map(|record| record.prefix_elapsed_ms)
            .collect::<Vec<_>>();
        let tail_opt = group
            .iter()
            .map(|record| record.tail_opt_len)
            .collect::<Vec<_>>();
        let tail_times = group
            .iter()
            .map(|record| record.tail_elapsed_ms)
            .collect::<Vec<_>>();
        let total_opt = group
            .iter()
            .map(|record| record.total_opt_len)
            .collect::<Vec<_>>();
        let final_mismatches = group
            .iter()
            .map(|record| record.final_mismatches)
            .collect::<Vec<_>>();
        let mut reasons = BTreeMap::new();
        for record in group {
            let reason = format!(
                "prefix:{};tail:{}",
                record.prefix_reason, record.tail_reason
            );
            *reasons.entry(reason).or_insert(0) += 1;
        }
        writeln!(
            file,
            "| {} | {} | {} | {} | {} | {}/{} | {}/{} | {}/{} | {} | {} | {} | {} | {} | {} | {} | {} | `{}` |",
            layout,
            difficulty,
            scramble_len,
            gate,
            group.len(),
            prefix_found,
            group.len(),
            tail_found,
            group.len(),
            total_found,
            group.len(),
            fmt_opt_f64(mean_usize(&prefix_len)),
            fmt_opt_f64(mean_usize(&prefix_mismatches)),
            fmt_opt_f64(mean_u128(&prefix_times)),
            fmt_opt_f64(mean_usize(&tail_opt)),
            fmt_opt_f64(mean_u128(&tail_times)),
            fmt_opt_f64(mean_usize(&total_opt)),
            fmt_opt_usize(percentile_usize(&total_opt, 0.95)),
            fmt_opt_f64(mean_usize(&final_mismatches)),
            join_counts(&reasons)
        )?;
    }

    Ok(())
}

fn moved_positions_text(permutation: &[u8]) -> String {
    permutation
        .iter()
        .enumerate()
        .filter_map(|(index, &source)| (source as usize != index).then_some(index.to_string()))
        .collect::<Vec<_>>()
        .join(";")
}

fn permutation_cycles(permutation: &[u8]) -> Vec<Vec<usize>> {
    let mut visited = vec![false; permutation.len()];
    let mut cycles = Vec::new();
    for start in 0..permutation.len() {
        if visited[start] {
            continue;
        }
        let mut current = start;
        let mut cycle = Vec::new();
        while !visited[current] {
            visited[current] = true;
            cycle.push(current);
            current = permutation[current] as usize;
        }
        if cycle.len() > 1 {
            cycles.push(cycle);
        }
    }
    cycles.sort_by(|left, right| {
        left.len()
            .cmp(&right.len())
            .then_with(|| left.first().cmp(&right.first()))
    });
    cycles
}

fn permutation_cycles_text(permutation: &[u8]) -> String {
    permutation_cycles(permutation)
        .iter()
        .map(|cycle| format!("({})", join_usize(cycle)))
        .collect::<Vec<_>>()
        .join(" ")
}

fn sticker_refs_text(puzzle: &Puzzle, permutation: &[u8]) -> String {
    permutation_cycles(permutation)
        .iter()
        .map(|cycle| {
            let refs = cycle
                .iter()
                .map(|&index| sticker_ref_text(puzzle, index))
                .collect::<Vec<_>>()
                .join(";");
            format!("({refs})")
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn sticker_ref_text(puzzle: &Puzzle, index: usize) -> String {
    let sticker = &puzzle.stickers[index];
    format!(
        "{}:{}:{}:{}#{}",
        sticker.face.name(),
        sticker.x,
        sticker.y,
        sticker.z,
        index
    )
}

fn write_beam_direction_survival_csv(
    path: &PathBuf,
    records: &[BeamDirectionSurvivalRecord],
) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(
        file,
        "layout,difficulty,target,operation_profile,scramble_len,iteration,seed,ariadne_unit_len,ariadne_first_move,beam_width,depth,survival,extinction_layer,direction_count,target_direction_entries,max_direction_entries,beam_size,nodes"
    )?;
    for r in records {
        writeln!(
            file,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            r.layout,
            r.difficulty,
            r.target.label(),
            r.operation_profile.label(),
            r.scramble_len,
            r.iteration,
            r.seed,
            r.ariadne_unit_len,
            csv(&r.ariadne_first_move),
            r.beam_width,
            r.depth,
            r.survival,
            fmt_opt_usize(r.extinction_layer),
            r.direction_count,
            r.target_direction_entries,
            r.max_direction_entries,
            r.beam_size,
            r.nodes
        )?;
    }
    Ok(())
}

fn write_beam_prefix_survival_csv(
    path: &PathBuf,
    records: &[BeamPrefixSurvivalRecord],
) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(
        file,
        "layout,difficulty,target,operation_profile,scramble_len,iteration,seed,ariadne_unit_len,beam_width,depth,target_prefix_len,path_prefix_alive,prefix_state_alive,max_matching_prefix,matching_states_count,strict_path_state_count,max_prefix_entries,mean_matching_prefix,beam_size,nodes"
    )?;
    for r in records {
        writeln!(
            file,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{:.6},{},{}",
            r.layout,
            r.difficulty,
            r.target.label(),
            r.operation_profile.label(),
            r.scramble_len,
            r.iteration,
            r.seed,
            r.ariadne_unit_len,
            r.beam_width,
            r.depth,
            r.target_prefix_len,
            r.path_prefix_alive,
            r.prefix_state_alive,
            r.max_matching_prefix,
            r.matching_states_count,
            r.strict_path_state_count,
            r.max_prefix_entries,
            r.mean_matching_prefix,
            r.beam_size,
            r.nodes
        )?;
    }
    Ok(())
}

fn write_backward_midpoint_audit_csv(
    path: &PathBuf,
    records: &[BackwardMidpointAuditRecord],
) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(
        file,
        "layout,difficulty,target,operation_profile,scramble_len,iteration,seed,ariadne_unit_len,midpoint_step,beam_width,depth,target_states,operation_count,candidate_hit,selected_hit,final_frontier_hit,first_candidate_layer,first_selected_layer,best_candidate_layer,best_candidate_rank,best_candidate_count,best_candidate_percentile,final_frontier_size,target_score,final_best_score,final_worst_score,nodes,elapsed_ms"
    )?;
    for r in records {
        writeln!(
            file,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{:.6},{},{},{},{},{}",
            r.layout,
            r.difficulty,
            r.target.label(),
            r.operation_profile.label(),
            r.scramble_len,
            r.iteration,
            r.seed,
            r.ariadne_unit_len,
            r.midpoint_step,
            r.beam_width,
            r.depth,
            r.target_states,
            r.operation_count,
            r.candidate_hit,
            r.selected_hit,
            r.final_frontier_hit,
            fmt_opt_usize(r.first_candidate_layer),
            fmt_opt_usize(r.first_selected_layer),
            fmt_opt_usize(r.best_candidate_layer),
            fmt_opt_usize(r.best_candidate_rank),
            fmt_opt_usize(r.best_candidate_count),
            r.best_candidate_percentile.unwrap_or(-1.0),
            r.final_frontier_size,
            r.target_score,
            fmt_opt_i32(r.final_best_score),
            fmt_opt_i32(r.final_worst_score),
            r.nodes,
            r.elapsed_ms
        )?;
    }
    Ok(())
}

fn write_backward_midpoint_audit_markdown(
    path: &PathBuf,
    options: &BenchOptions,
    records: &[BackwardMidpointAuditRecord],
) -> io::Result<()> {
    let mut groups: BTreeMap<
        (
            LayoutId,
            Difficulty,
            usize,
            String,
            String,
            usize,
            usize,
            usize,
        ),
        BackwardMidpointGroupStats,
    > = BTreeMap::new();

    for record in records {
        let group = groups
            .entry((
                record.layout,
                record.difficulty,
                record.scramble_len,
                record.target.label().to_string(),
                record.operation_profile.label().to_string(),
                record.beam_width,
                record.depth,
                record.midpoint_step,
            ))
            .or_default();
        group.total += 1;
        if record.candidate_hit {
            group.candidate_hits += 1;
        }
        if record.selected_hit {
            group.selected_hits += 1;
        }
        if record.final_frontier_hit {
            group.final_hits += 1;
        }
        group.ariadne_lens.push(record.ariadne_unit_len);
        group.midpoint_steps.push(record.midpoint_step);
        if let Some(value) = record.first_candidate_layer {
            group.first_candidate_layers.push(value);
        }
        if let Some(value) = record.first_selected_layer {
            group.first_selected_layers.push(value);
        }
        if let Some(value) = record.best_candidate_percentile {
            group.best_candidate_percentiles.push(value);
        }
        group.final_frontier_sizes.push(record.final_frontier_size);
        group.target_scores.push(record.target_score);
        if let Some(value) = record.final_best_score {
            group.final_best_scores.push(value);
        }
        if let Some(value) = record.final_worst_score {
            group.final_worst_scores.push(value);
        }
        group.nodes.push(record.nodes);
        group.elapsed_ms.push(record.elapsed_ms);
    }

    let mut file = File::create(path)?;
    writeln!(file, "# SkimmIQ Backward Midpoint Audit")?;
    writeln!(file)?;
    writeln!(file, "- target: `{}`", options.solver.target_mode.label())?;
    writeln!(
        file,
        "- operation_profile: `{}`",
        options.solver.operation_profile.label()
    )?;
    writeln!(
        file,
        "- scramble_profile: `{}`",
        options.scramble_profile.label()
    )?;
    writeln!(file, "- layouts: `{}`", join_display(&options.layouts))?;
    writeln!(
        file,
        "- difficulties: `{}`",
        join_display(&options.difficulties)
    )?;
    writeln!(file, "- scrambles: `{:?}`", options.scramble_lengths)?;
    writeln!(file, "- iterations: `{}`", iteration_label(options))?;
    writeln!(file, "- seed: `{}`", options.seed)?;
    if let Some(record) = records.first() {
        writeln!(file, "- beam_width: `{}`", record.beam_width)?;
        writeln!(file, "- depth: `{}`", record.depth)?;
        writeln!(file, "- midpoint_step: `{}`", record.midpoint_step)?;
        writeln!(file, "- target_states: `{}`", record.target_states)?;
        writeln!(file, "- operation_count: `{}`", record.operation_count)?;
    }
    writeln!(
        file,
        "- pattern_db_enabled: `{}`",
        options.solver.pattern_db_enabled
    )?;
    writeln!(file)?;
    writeln!(
        file,
        "`Candidate hit` means the Ariadne midpoint state was generated by the backward beam before truncation. `Selected hit` means it survived truncation in at least one layer. `Final hit` means it is present in the final frontier after the requested depth."
    )?;
    writeln!(file)?;
    writeln!(file, "## Summary")?;
    writeln!(file)?;
    writeln!(
        file,
        "| Layout | Difficulty | Scramble | Target | Profile | Width | Depth | Step | Candidate Hit | Selected Hit | Final Hit | Mean Ariadne | Mean Best Candidate % | Mean First Candidate Layer | Mean First Selected Layer | Mean Target Score | Mean Final Best Score | Mean Final Worst Score | Mean Time ms | Mean Nodes |"
    )?;
    writeln!(
        file,
        "|---|---|---:|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|"
    )?;
    for (
        (layout, difficulty, scramble_len, target, profile, beam_width, depth, midpoint_step),
        group,
    ) in &groups
    {
        writeln!(
            file,
            "| {} | {} | {} | `{}` | `{}` | {} | {} | {} | {}/{} | {}/{} | {}/{} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            layout,
            difficulty,
            scramble_len,
            target,
            profile,
            beam_width,
            depth,
            midpoint_step,
            group.candidate_hits,
            group.total,
            group.selected_hits,
            group.total,
            group.final_hits,
            group.total,
            fmt_opt_f64(mean_usize(&group.ariadne_lens)),
            fmt_opt_f64(mean_f64(&group.best_candidate_percentiles)),
            fmt_opt_f64(mean_usize(&group.first_candidate_layers)),
            fmt_opt_f64(mean_usize(&group.first_selected_layers)),
            fmt_opt_f64(mean_i32(&group.target_scores)),
            fmt_opt_f64(mean_i32(&group.final_best_scores)),
            fmt_opt_f64(mean_i32(&group.final_worst_scores)),
            fmt_opt_f64(mean_u128(&group.elapsed_ms)),
            fmt_opt_f64(mean_u64(&group.nodes))
        )?;
    }

    writeln!(file)?;
    writeln!(file, "## Per Iteration")?;
    writeln!(file)?;
    writeln!(
        file,
        "| Iter | Seed | Ariadne | Step | Candidate | Selected | Final | First Candidate Layer | First Selected Layer | Best Rank | Best % | Final Frontier | Target Score | Final Best | Final Worst | Time ms |"
    )?;
    writeln!(
        file,
        "|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|"
    )?;
    for record in records {
        writeln!(
            file,
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            record.iteration,
            record.seed,
            record.ariadne_unit_len,
            record.midpoint_step,
            record.candidate_hit,
            record.selected_hit,
            record.final_frontier_hit,
            fmt_opt_usize(record.first_candidate_layer),
            fmt_opt_usize(record.first_selected_layer),
            fmt_opt_usize(record.best_candidate_rank),
            fmt_opt_f64(record.best_candidate_percentile),
            record.final_frontier_size,
            record.target_score,
            fmt_opt_i32(record.final_best_score),
            fmt_opt_i32(record.final_worst_score),
            record.elapsed_ms
        )?;
    }

    Ok(())
}

fn write_ariadne_check_markdown(
    path: &PathBuf,
    options: &BenchOptions,
    records: &[AriadneCheckRecord],
) -> io::Result<()> {
    let mut groups: BTreeMap<(LayoutId, Difficulty, usize), AriadneCheckGroupStats> =
        BTreeMap::new();
    for record in records {
        let group = groups
            .entry((record.layout, record.difficulty, record.scramble_len))
            .or_default();
        group.total += 1;
        if record.exact_solved {
            group.exact_ok += 1;
        }
        if record.uniform_solved {
            group.uniform_ok += 1;
        }
        if record.android_solved {
            group.android_ok += 1;
        }
        group.stack_lens.push(record.ariadne_stack_len);
        group.unit_lens.push(record.ariadne_unit_len);
    }

    let mut file = File::create(path)?;
    writeln!(file, "# SkimmIQ Ariadne Check")?;
    writeln!(file)?;
    writeln!(file, "- layouts: `{}`", join_display(&options.layouts))?;
    writeln!(
        file,
        "- difficulties: `{}`",
        join_display(&options.difficulties)
    )?;
    writeln!(file, "- scrambles: `{:?}`", options.scramble_lengths)?;
    writeln!(
        file,
        "- scramble_profile: `{}`",
        options.scramble_profile.label()
    )?;
    writeln!(file, "- iterations: `{}`", options.iterations)?;
    writeln!(file, "- iteration_start: `{}`", options.iteration_start + 1)?;
    writeln!(file, "- seed: `{}`", options.seed)?;
    writeln!(file, "- include_scripts: `{}`", options.include_scripts)?;
    writeln!(file)?;
    writeln!(
        file,
        "`Stack len` is the historical Ariadne move count: one reduced tape move can represent multiple unit shifts. `Unit len` expands that reduced plan into legal `+/-` unit moves used by the solver."
    )?;
    writeln!(file)?;
    writeln!(
        file,
        "| Layout | Difficulty | Scramble | Exact OK | Android OK | Mean Stack | Mean Unit | P95 Unit | Max Unit | Unit/Stack |"
    )?;
    writeln!(file, "|---|---|---:|---:|---:|---:|---:|---:|---:|---:|")?;

    for ((layout, difficulty, scramble_len), group) in groups {
        let mean_stack = mean_usize(&group.stack_lens);
        let mean_unit = mean_usize(&group.unit_lens);
        let p95_unit = percentile_usize(&group.unit_lens, 0.95);
        let max_unit = group.unit_lens.iter().copied().max();
        let unit_stack_ratio = match (mean_unit, mean_stack) {
            (Some(unit), Some(stack)) if stack > 0.0 => Some(unit / stack),
            _ => None,
        };
        writeln!(
            file,
            "| {} | {} | {} | {}/{} | {}/{} | {} | {} | {} | {} | {} |",
            layout,
            difficulty,
            scramble_len,
            group.exact_ok,
            group.total,
            group.android_ok,
            group.total,
            fmt_opt_f64(mean_stack),
            fmt_opt_f64(mean_unit),
            fmt_opt_usize(p95_unit),
            fmt_opt_usize(max_unit),
            fmt_opt_f64(unit_stack_ratio)
        )?;
    }

    Ok(())
}

fn write_heuristic_audit_markdown(
    path: &PathBuf,
    options: &BenchOptions,
    records: &[HeuristicAuditRecord],
) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(file, "# SkimmIQ Heuristic Audit")?;
    writeln!(file)?;
    writeln!(file, "- target: `{}`", options.solver.target_mode.label())?;
    writeln!(file, "- layouts: `{}`", join_display(&options.layouts))?;
    writeln!(
        file,
        "- projections: `{}`",
        join_projection_kinds(&options.projection_kinds)
    )?;
    writeln!(
        file,
        "- difficulties: `{}`",
        join_display(&options.difficulties)
    )?;
    writeln!(file, "- audit_depth: `{}`", options.solver.pattern_db_depth)?;
    writeln!(file, "- max_nodes: `{}`", options.solver.max_nodes)?;
    writeln!(file, "- time_limit_ms: `{}`", options.solver.time_limit_ms)?;
    writeln!(file)?;
    writeln!(
        file,
        "| Layout | Difficulty | Target | Depth | States | Histogram Keys | Histogram Ratio | Canonical Keys | Canonical Ratio | Pearson(score,distance) | Nodes | Time ms | Reason |"
    )?;
    writeln!(
        file,
        "|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|"
    )?;
    for r in records {
        writeln!(
            file,
            "| {} | {} | `{}` | {} | {} | {} | {:.4} | {} | {:.4} | {:.4} | {} | {} | `{}` |",
            r.layout,
            r.difficulty,
            r.target.label(),
            r.depth,
            r.states,
            r.histogram_keys,
            ratio(r.histogram_keys, r.states),
            r.canonical_histogram_keys,
            ratio(r.canonical_histogram_keys, r.states),
            r.pearson_score_distance,
            r.nodes,
            r.elapsed_ms,
            r.reason
        )?;
    }

    writeln!(file)?;
    writeln!(file, "## Depth Buckets")?;
    writeln!(file)?;
    for r in records {
        writeln!(
            file,
            "### {} {} `{}`",
            r.layout,
            r.difficulty,
            r.target.label()
        )?;
        writeln!(file)?;
        writeln!(file, "| Distance | States | Mean Score |")?;
        writeln!(file, "|---:|---:|---:|")?;
        for distance in 0..=r.depth {
            let count = r.depth_counts.get(distance).copied().unwrap_or_default();
            let mean_score = r
                .mean_score_by_depth
                .get(distance)
                .copied()
                .unwrap_or_default();
            writeln!(file, "| {} | {} | {:.3} |", distance, count, mean_score)?;
        }
        writeln!(file)?;
    }

    Ok(())
}

fn write_projection_audit_markdown(
    path: &PathBuf,
    options: &BenchOptions,
    records: &[ProjectionAuditRecord],
) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(file, "# SkimmIQ Projection Audit")?;
    writeln!(file)?;
    writeln!(file, "- target: `{}`", options.solver.target_mode.label())?;
    writeln!(file, "- layouts: `{}`", join_display(&options.layouts))?;
    writeln!(
        file,
        "- difficulties: `{}`",
        join_display(&options.difficulties)
    )?;
    writeln!(file, "- audit_depth: `{}`", options.solver.pattern_db_depth)?;
    writeln!(file, "- max_nodes: `{}`", options.solver.max_nodes)?;
    writeln!(file, "- time_limit_ms: `{}`", options.solver.time_limit_ms)?;
    writeln!(file)?;
    writeln!(
        file,
        "`Key ratio` says how many distinct abstract states survive the projection. `Depth span` shows how far apart real BFS distances can be inside one projected bucket; lower is cleaner."
    )?;
    writeln!(file)?;
    writeln!(
        file,
        "| Layout | Difficulty | Target | Projection | Depth | States | Keys | Key Ratio | Mean Span | P95 Span | Max Span | Mean Bucket | P95 Bucket | Max Bucket | Time ms | Reason |"
    )?;
    writeln!(
        file,
        "|---|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|"
    )?;
    for r in records {
        writeln!(
            file,
            "| {} | {} | `{}` | `{}` | {} | {} | {} | {:.4} | {:.2} | {} | {} | {:.2} | {} | {} | {} | `{}` |",
            r.layout,
            r.difficulty,
            r.target.label(),
            r.projection.label(),
            r.depth,
            r.states,
            r.keys,
            ratio(r.keys, r.states),
            r.mean_depth_span,
            r.p95_depth_span,
            r.max_depth_span,
            r.mean_states_per_key,
            r.p95_states_per_key,
            r.max_states_per_key,
            r.elapsed_ms,
            r.reason
        )?;
    }

    Ok(())
}

fn write_axis_ring_pdb_audit_markdown(
    path: &PathBuf,
    options: &BenchOptions,
    records: &[AxisRingPdbRecord],
) -> io::Result<()> {
    let mut groups: BTreeMap<(LayoutId, Difficulty, usize), AxisRingPdbGroupStats> =
        BTreeMap::new();
    for record in records {
        let group = groups
            .entry((record.layout, record.difficulty, record.scramble_len))
            .or_default();
        group.total += 1;
        group.ariadne_lens.push(record.ariadne_unit_len);
        group.path_hit_counts.push(record.hit_count);
        if record.start_hit_distance.is_some() {
            group.start_hits += 1;
        }
        if record.first_hit_step.is_some() {
            group.first_hits += 1;
        }
        if let Some(value) = record.first_hit_remaining_to_solved {
            group.first_remaining.push(value);
        }
        if let Some(value) = record.best_hit_remaining_to_solved {
            group.best_remaining.push(value);
        }
        if let Some(value) = record.best_prefix_to_axis_ring {
            group.best_prefix_to_axis.push(value);
        }
        if let Some(value) = record.estimated_total_with_max_suffix {
            group.estimated_totals.push(value);
        }
        group
            .path_distance_remaining_pearsons
            .push(record.path_distance_remaining_pearson);
    }

    let mut file = File::create(path)?;
    writeln!(file, "# SkimmIQ Axis-Ring Multi-Start PDB Audit")?;
    writeln!(file)?;
    writeln!(file, "- seed_source: `{}`", options.pdb_seed_source.label())?;
    writeln!(file, "- target: `{}`", options.solver.target_mode.label())?;
    writeln!(file, "- layouts: `{}`", join_display(&options.layouts))?;
    writeln!(
        file,
        "- difficulties: `{}`",
        join_display(&options.difficulties)
    )?;
    writeln!(file, "- scrambles: `{:?}`", options.scramble_lengths)?;
    writeln!(
        file,
        "- scramble_profile: `{}`",
        options.scramble_profile.label()
    )?;
    writeln!(
        file,
        "- axis_table_depth: `{}`",
        options.axis_ring_rescue_table_depth
    )?;
    writeln!(
        file,
        "- target_expand_depth: `{}`",
        options.axis_ring_rescue_expand_depth
    )?;
    writeln!(file, "- pdb_seed_count: `{}`", options.pdb_seed_count)?;
    writeln!(
        file,
        "- pdb_seed_steps: `{}..={}`",
        options.pdb_seed_step_start, options.pdb_seed_step_end
    )?;
    writeln!(
        file,
        "- pdb_random_walk: `{}..={}`",
        options.pdb_random_walk_min, options.pdb_random_walk_max
    )?;
    writeln!(file, "- pdb_depth: `{}`", options.solver.pattern_db_depth)?;
    writeln!(file, "- max_nodes: `{}`", options.solver.max_nodes)?;
    writeln!(file, "- time_limit_ms: `{}`", options.solver.time_limit_ms)?;
    if let Some(first) = records.first() {
        writeln!(file, "- target_states: `{}`", first.target_states)?;
        writeln!(file, "- pdb_states: `{}`", first.pdb_states)?;
        writeln!(
            file,
            "- depth_counts: `{}`",
            join_usize_counts(&first.depth_counts)
        )?;
        writeln!(file, "- build_nodes: `{}`", first.build_nodes)?;
        writeln!(file, "- build_elapsed_ms: `{}`", first.build_elapsed_ms)?;
        writeln!(file, "- build_reason: `{}`", first.reason)?;
        writeln!(file, "- max_axis_suffix: `{}`", first.max_axis_suffix)?;
    }
    writeln!(file)?;
    writeln!(
        file,
        "`First remaining` is how many Ariadne unit moves were still left when the path first entered the axis-ring PDB. Larger values mean the PDB sees useful structure earlier."
    )?;
    writeln!(file)?;
    writeln!(
        file,
        "| Layout | Difficulty | Scramble | Path Hit | Start Hit | Mean Ariadne | Mean Hits/Path | Mean First Remaining | Mean Best Remaining | Mean Best Prefix To Seed | Mean Est Total + Max Suffix | Mean Pearson(dist,remaining) |"
    )?;
    writeln!(
        file,
        "|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|"
    )?;
    for ((layout, difficulty, scramble_len), group) in groups {
        let mean_hits = mean_usize(&group.path_hit_counts);
        let mean_ariadne = mean_usize(&group.ariadne_lens);
        let mean_first_remaining = mean_usize(&group.first_remaining);
        let mean_best_remaining = mean_usize(&group.best_remaining);
        let mean_best_prefix = mean_usize(&group.best_prefix_to_axis);
        let mean_est_total = mean_usize(&group.estimated_totals);
        let mean_pearson = mean_f64(&group.path_distance_remaining_pearsons);
        writeln!(
            file,
            "| {} | {} | {} | {}/{} | {}/{} | {} | {} | {} | {} | {} | {} | {} |",
            layout,
            difficulty,
            scramble_len,
            group.first_hits,
            group.total,
            group.start_hits,
            group.total,
            fmt_opt_f64(mean_ariadne),
            fmt_opt_f64(mean_hits),
            fmt_opt_f64(mean_first_remaining),
            fmt_opt_f64(mean_best_remaining),
            fmt_opt_f64(mean_best_prefix),
            fmt_opt_f64(mean_est_total),
            fmt_opt_f64(mean_pearson)
        )?;
    }

    Ok(())
}

fn write_move_delta_audit_markdown(
    path: &PathBuf,
    options: &BenchOptions,
    records: &[MoveDeltaAuditRecord],
) -> io::Result<()> {
    let mut groups: BTreeMap<
        (LayoutId, Difficulty, usize, &'static str, &'static str),
        MoveDeltaAuditGroupStats,
    > = BTreeMap::new();

    for record in records {
        for bucket in [record.step_bucket, "all"] {
            let group = groups
                .entry((
                    record.layout,
                    record.difficulty,
                    record.scramble_len,
                    record.feature,
                    bucket,
                ))
                .or_default();
            group.total += 1;
            group.candidate_counts.push(record.candidate_count);
            group.first_match_counts.push(record.first_match_count);
            group.raw_percentiles.push(record.raw_percentile);
            group
                .first_match_percentiles
                .push(record.first_match_percentile);
        }
    }

    let mut file = File::create(path)?;
    writeln!(file, "# SkimmIQ Move Delta Audit")?;
    writeln!(file)?;
    writeln!(file, "- target: `{}`", options.solver.target_mode.label())?;
    writeln!(
        file,
        "- operation_profile: `{}`",
        options.solver.operation_profile.label()
    )?;
    writeln!(
        file,
        "- scramble_profile: `{}`",
        options.scramble_profile.label()
    )?;
    writeln!(file, "- layouts: `{}`", join_display(&options.layouts))?;
    writeln!(
        file,
        "- difficulties: `{}`",
        join_display(&options.difficulties)
    )?;
    writeln!(file, "- scrambles: `{:?}`", options.scramble_lengths)?;
    writeln!(file, "- iterations: `{}`", iteration_label(options))?;
    writeln!(file, "- seed: `{}`", options.seed)?;
    writeln!(file)?;
    writeln!(
        file,
        "`Raw %` is the rank percentile of the exact single Ariadne unit move among all operations. `First-match %` is the best percentile among operations whose first unit move matches Ariadne. Lower is better: `0%` is top-ranked, `50%` is random-like."
    )?;
    writeln!(file)?;
    writeln!(
        file,
        "| Layout | Difficulty | Scramble | Feature | Steps | Rows | Mean Raw % | Raw P25 | Raw P50 | Raw P75 | Raw P95 | Mean First % | First P50 | First P95 | Mean Candidates | Mean First-Match Ops |"
    )?;
    writeln!(
        file,
        "|---|---|---:|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|"
    )?;

    for ((layout, difficulty, scramble_len, feature, bucket), group) in groups {
        writeln!(
            file,
            "| {} | {} | {} | `{}` | `{}` | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            layout,
            difficulty,
            scramble_len,
            feature,
            bucket,
            group.total,
            fmt_opt_f64(mean_f64(&group.raw_percentiles)),
            fmt_opt_f64(percentile_f64(&group.raw_percentiles, 0.25)),
            fmt_opt_f64(percentile_f64(&group.raw_percentiles, 0.50)),
            fmt_opt_f64(percentile_f64(&group.raw_percentiles, 0.75)),
            fmt_opt_f64(percentile_f64(&group.raw_percentiles, 0.95)),
            fmt_opt_f64(mean_f64(&group.first_match_percentiles)),
            fmt_opt_f64(percentile_f64(&group.first_match_percentiles, 0.50)),
            fmt_opt_f64(percentile_f64(&group.first_match_percentiles, 0.95)),
            fmt_opt_f64(mean_usize(&group.candidate_counts)),
            fmt_opt_f64(mean_usize(&group.first_match_counts))
        )?;
    }

    Ok(())
}

fn write_feature_cost_markdown(
    path: &PathBuf,
    options: &BenchOptions,
    records: &[FeatureCostRecord],
) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(file, "# SkimmIQ Feature Cost Audit")?;
    writeln!(file)?;
    writeln!(file, "- target: `{}`", options.solver.target_mode.label())?;
    writeln!(
        file,
        "- scramble_profile: `{}`",
        options.scramble_profile.label()
    )?;
    writeln!(file, "- layouts: `{}`", join_display(&options.layouts))?;
    writeln!(
        file,
        "- difficulties: `{}`",
        join_display(&options.difficulties)
    )?;
    writeln!(file, "- scrambles: `{:?}`", options.scramble_lengths)?;
    writeln!(file, "- states: `{}`", iteration_label(options))?;
    writeln!(file, "- feature_repeats: `{}`", options.feature_repeats)?;
    writeln!(file, "- seed: `{}`", options.seed)?;
    writeln!(file)?;
    writeln!(
        file,
        "`Mean ns` is the average wall-clock nanoseconds for one feature evaluation on a scrambled state. The checksum is only a guard against dead-code elimination."
    )?;
    writeln!(file)?;
    writeln!(
        file,
        "| Layout | Difficulty | Scramble | Feature | States | Repeats | Evals | Mean ns | Total ms | Checksum |"
    )?;
    writeln!(file, "|---|---|---:|---|---:|---:|---:|---:|---:|---:|")?;
    for r in records {
        writeln!(
            file,
            "| {} | {} | {} | `{}` | {} | {} | {} | {:.1} | {:.3} | {} |",
            r.layout,
            r.difficulty,
            r.scramble_len,
            r.feature,
            r.states,
            r.repeats,
            r.evals,
            r.mean_ns,
            r.total_ns as f64 / 1_000_000.0,
            r.checksum
        )?;
    }

    Ok(())
}

fn write_commutator_scan_markdown(
    path: &PathBuf,
    options: &BenchOptions,
    records: &[CommutatorScanRecord],
) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(file, "# SkimmIQ Commutator Scan")?;
    writeln!(file)?;
    writeln!(file, "- layouts: `{}`", join_display(&options.layouts))?;
    writeln!(
        file,
        "- difficulties: `{}`",
        join_display(&options.difficulties)
    )?;
    writeln!(file, "- max_len(A/B): `{}`", options.commutator_max_len)?;
    writeln!(file, "- top candidates: `{}`", options.commutator_top)?;
    writeln!(file, "- max_nodes: `{}`", options.solver.max_nodes)?;
    writeln!(file, "- time_limit_ms: `{}`", options.solver.time_limit_ms)?;
    writeln!(file)?;
    writeln!(
        file,
        "This scan classifies commutators `A B A^-1 B^-1` by permutation cycle type. A clean 3-cycle is the main reduction-solver primitive; a double transposition is useful for parity handling. A robust commutator solver should first be judged by coverage/guarantee, not by beating the current E-classic beam mean."
    )?;
    writeln!(file)?;
    writeln!(
        file,
        "| Layout | Difficulty | Max Len | Sequences | Pairs | Unique Perms | Identity | Min Support | Clean 3-cycles | Double Transpositions | Time ms | Reason |"
    )?;
    writeln!(
        file,
        "|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|"
    )?;
    for r in records {
        writeln!(
            file,
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | `{}` |",
            r.layout,
            r.difficulty,
            r.max_len,
            r.sequences,
            r.pairs_examined,
            r.unique_permutations,
            r.identity_count,
            r.min_support,
            r.clean_3_cycles,
            r.double_transpositions,
            r.elapsed_ms,
            r.reason
        )?;
    }

    for r in records {
        writeln!(file)?;
        writeln!(file, "## {} {}", r.layout, r.difficulty)?;
        writeln!(file)?;
        writeln!(file, "### Cycle-Type Histogram")?;
        writeln!(file)?;
        writeln!(file, "| Cycle Type | Count |")?;
        writeln!(file, "|---|---:|")?;
        for (cycle_type, count) in &r.histogram {
            writeln!(file, "| `{}` | {} |", cycle_type, count)?;
        }
        writeln!(file)?;
        writeln!(file, "### Best Candidates")?;
        writeln!(file)?;
        writeln!(
            file,
            "| Rank | Support | Cycle Type | Cycle Lengths | A | B | Full Commutator |"
        )?;
        writeln!(file, "|---:|---:|---|---|---|---|---|")?;
        for (rank, candidate) in r.top.iter().enumerate() {
            writeln!(
                file,
                "| {} | {} | `{}` | `{}` | `{}` | `{}` | `{}` |",
                rank + 1,
                candidate.support,
                candidate.cycle_type,
                join_usize(&candidate.cycle_lengths),
                puzzle_sequence_text_for_record(r.layout, r.difficulty, &candidate.a),
                puzzle_sequence_text_for_record(r.layout, r.difficulty, &candidate.b),
                puzzle_sequence_text_for_record(r.layout, r.difficulty, &candidate.moves)
            )?;
        }
    }

    Ok(())
}

fn write_beam_direction_survival_markdown(
    path: &PathBuf,
    options: &BenchOptions,
    records: &[BeamDirectionSurvivalRecord],
) -> io::Result<()> {
    let mut depth_groups: BTreeMap<
        (LayoutId, Difficulty, usize, String, String, usize, usize),
        BeamDirectionSurvivalGroupStats,
    > = BTreeMap::new();
    let mut iteration_extinctions: BTreeMap<
        (
            LayoutId,
            Difficulty,
            usize,
            String,
            String,
            usize,
            usize,
            u64,
        ),
        Option<usize>,
    > = BTreeMap::new();

    for record in records {
        let depth_group = depth_groups
            .entry((
                record.layout,
                record.difficulty,
                record.scramble_len,
                record.target.label().to_string(),
                record.operation_profile.label().to_string(),
                record.beam_width,
                record.depth,
            ))
            .or_default();
        depth_group.total += 1;
        if record.survival {
            depth_group.alive += 1;
        }
        depth_group.direction_counts.push(record.direction_count);
        depth_group
            .target_direction_entries
            .push(record.target_direction_entries);
        depth_group
            .max_direction_entries
            .push(record.max_direction_entries);
        depth_group.beam_sizes.push(record.beam_size);
        depth_group.nodes.push(record.nodes);

        let iteration_key = (
            record.layout,
            record.difficulty,
            record.scramble_len,
            record.target.label().to_string(),
            record.operation_profile.label().to_string(),
            record.beam_width,
            record.iteration,
            record.seed,
        );
        let entry = iteration_extinctions.entry(iteration_key).or_insert(None);
        if entry.is_none() {
            *entry = record.extinction_layer;
        }
    }

    let mut extinction_groups: BTreeMap<
        (LayoutId, Difficulty, usize, String, String, usize),
        BeamDirectionExtinctionStats,
    > = BTreeMap::new();
    for ((layout, difficulty, scramble_len, target, profile, beam_width, _, _), extinction_layer) in
        iteration_extinctions
    {
        let extinction_group = extinction_groups
            .entry((
                layout,
                difficulty,
                scramble_len,
                target,
                profile,
                beam_width,
            ))
            .or_default();
        extinction_group.total += 1;
        if let Some(layer) = extinction_layer {
            extinction_group.extinct += 1;
            if layer <= 2 {
                extinction_group.extinct_by_2 += 1;
            }
            if layer <= 5 {
                extinction_group.extinct_by_5 += 1;
            }
            extinction_group.extinction_layers.push(layer);
        }
    }

    let mut file = File::create(path)?;
    writeln!(file, "# SkimmIQ Beam Direction Survival Audit")?;
    writeln!(file)?;
    writeln!(file, "- target: `{}`", options.solver.target_mode.label())?;
    writeln!(
        file,
        "- operation_profile: `{}`",
        options.solver.operation_profile.label()
    )?;
    writeln!(
        file,
        "- scramble_profile: `{}`",
        options.scramble_profile.label()
    )?;
    writeln!(file, "- layouts: `{}`", join_display(&options.layouts))?;
    writeln!(
        file,
        "- difficulties: `{}`",
        join_display(&options.difficulties)
    )?;
    writeln!(file, "- scrambles: `{:?}`", options.scramble_lengths)?;
    writeln!(file, "- iterations: `{}`", iteration_label(options))?;
    writeln!(file, "- seed: `{}`", options.seed)?;
    if let Some(record) = records.first() {
        writeln!(file, "- beam_width: `{}`", record.beam_width)?;
    }
    writeln!(
        file,
        "- corridor_diversity: `{}` prefix_len=`{}` quota_percent=`{}`",
        options.solver.corridor_diversity_enabled,
        options.solver.corridor_prefix_len,
        options.solver.corridor_quota_percent
    )?;
    writeln!(
        file,
        "- pattern_db_enabled: `{}`",
        options.solver.pattern_db_enabled
    )?;
    writeln!(file)?;
    writeln!(
        file,
        "This audit tracks the first unit move used by each beam state from the scrambled start. `Survival %` says whether Ariadne's first unit move is still represented after a beam layer. Layers are reported after expansion, so layer `1` means after the first macro expansion."
    )?;
    writeln!(file)?;
    writeln!(file, "## Layer Survival")?;
    writeln!(file)?;
    writeln!(
        file,
        "| Layout | Difficulty | Scramble | Target | Profile | Width | Layer | Survival | Survival % | Mean Directions | Mean Target Entries | Mean Max Direction Entries | Mean Beam Size | Mean Nodes |"
    )?;
    writeln!(
        file,
        "|---|---|---:|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|"
    )?;
    for ((layout, difficulty, scramble_len, target, profile, beam_width, depth), group) in
        &depth_groups
    {
        let survival_pct = if group.total == 0 {
            0.0
        } else {
            group.alive as f64 * 100.0 / group.total as f64
        };
        writeln!(
            file,
            "| {} | {} | {} | `{}` | `{}` | {} | {} | {}/{} | {:.1} | {} | {} | {} | {} | {} |",
            layout,
            difficulty,
            scramble_len,
            target,
            profile,
            beam_width,
            depth,
            group.alive,
            group.total,
            survival_pct,
            fmt_opt_f64(mean_usize(&group.direction_counts)),
            fmt_opt_f64(mean_usize(&group.target_direction_entries)),
            fmt_opt_f64(mean_usize(&group.max_direction_entries)),
            fmt_opt_f64(mean_usize(&group.beam_sizes)),
            fmt_opt_f64(mean_u64(&group.nodes))
        )?;
    }

    writeln!(file)?;
    writeln!(file, "## Extinction Summary")?;
    writeln!(file)?;
    writeln!(
        file,
        "| Layout | Difficulty | Scramble | Target | Profile | Width | Iterations | Extinct | Extinct <=2 | Extinct <=5 | Never Extinct | Mean Extinction Layer | P50 Extinction |"
    )?;
    writeln!(
        file,
        "|---|---|---:|---|---|---:|---:|---:|---:|---:|---:|---:|---:|"
    )?;
    for ((layout, difficulty, scramble_len, target, profile, beam_width), group) in
        extinction_groups
    {
        let never_extinct = group.total.saturating_sub(group.extinct);
        writeln!(
            file,
            "| {} | {} | {} | `{}` | `{}` | {} | {} | {} | {} | {} | {} | {} | {} |",
            layout,
            difficulty,
            scramble_len,
            target,
            profile,
            beam_width,
            group.total,
            group.extinct,
            group.extinct_by_2,
            group.extinct_by_5,
            never_extinct,
            fmt_opt_f64(mean_usize(&group.extinction_layers)),
            fmt_opt_usize(percentile_usize(&group.extinction_layers, 0.50))
        )?;
    }

    Ok(())
}

fn write_beam_prefix_survival_markdown(
    path: &PathBuf,
    options: &BenchOptions,
    records: &[BeamPrefixSurvivalRecord],
) -> io::Result<()> {
    let mut groups: BTreeMap<
        (LayoutId, Difficulty, usize, String, String, usize, usize),
        BeamPrefixSurvivalGroupStats,
    > = BTreeMap::new();

    for record in records {
        let group = groups
            .entry((
                record.layout,
                record.difficulty,
                record.scramble_len,
                record.target.label().to_string(),
                record.operation_profile.label().to_string(),
                record.beam_width,
                record.depth,
            ))
            .or_default();
        group.total += 1;
        if record.path_prefix_alive {
            group.path_prefix_alive += 1;
        }
        if record.prefix_state_alive {
            group.prefix_state_alive += 1;
        }
        group.max_matching_prefixes.push(record.max_matching_prefix);
        group
            .matching_states_counts
            .push(record.matching_states_count);
        group
            .strict_path_state_counts
            .push(record.strict_path_state_count);
        group.max_prefix_entries.push(record.max_prefix_entries);
        group
            .mean_matching_prefixes
            .push(record.mean_matching_prefix);
        group.beam_sizes.push(record.beam_size);
        group.nodes.push(record.nodes);
    }

    let mut file = File::create(path)?;
    writeln!(file, "# SkimmIQ Beam Prefix Survival Audit")?;
    writeln!(file)?;
    writeln!(file, "- target: `{}`", options.solver.target_mode.label())?;
    writeln!(
        file,
        "- operation_profile: `{}`",
        options.solver.operation_profile.label()
    )?;
    writeln!(
        file,
        "- scramble_profile: `{}`",
        options.scramble_profile.label()
    )?;
    writeln!(file, "- layouts: `{}`", join_display(&options.layouts))?;
    writeln!(
        file,
        "- difficulties: `{}`",
        join_display(&options.difficulties)
    )?;
    writeln!(file, "- scrambles: `{:?}`", options.scramble_lengths)?;
    writeln!(file, "- iterations: `{}`", iteration_label(options))?;
    writeln!(file, "- seed: `{}`", options.seed)?;
    if let Some(record) = records.first() {
        writeln!(file, "- beam_width: `{}`", record.beam_width)?;
    }
    writeln!(
        file,
        "- corridor_diversity: `{}` prefix_len=`{}` quota_percent=`{}`",
        options.solver.corridor_diversity_enabled,
        options.solver.corridor_prefix_len,
        options.solver.corridor_quota_percent
    )?;
    writeln!(
        file,
        "- pattern_db_enabled: `{}`",
        options.solver.pattern_db_enabled
    )?;
    writeln!(file)?;
    writeln!(
        file,
        "`Path prefix alive` means at least one surviving beam path begins with the first k Ariadne unit moves, where k is the layer number. `Prefix state alive` means the exact color state reached by the first k Ariadne unit moves is present in the beam, regardless of the path that reached it. Because operations can be macros, `Max matching prefix` can be greater than the layer number."
    )?;
    writeln!(file)?;
    writeln!(
        file,
        "| Layout | Difficulty | Scramble | Target | Profile | Width | Layer | Path Prefix Alive | Prefix State Alive | Mean Max Prefix | P50 Max Prefix | Mean Matching States >=k | Mean Strict Path States | Mean Max Prefix Entries | Mean Path Prefix | Mean Beam Size | Mean Nodes |"
    )?;
    writeln!(
        file,
        "|---|---|---:|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|"
    )?;
    for ((layout, difficulty, scramble_len, target, profile, beam_width, depth), group) in groups {
        writeln!(
            file,
            "| {} | {} | {} | `{}` | `{}` | {} | {} | {}/{} | {}/{} | {} | {} | {} | {} | {} | {} | {} | {} |",
            layout,
            difficulty,
            scramble_len,
            target,
            profile,
            beam_width,
            depth,
            group.path_prefix_alive,
            group.total,
            group.prefix_state_alive,
            group.total,
            fmt_opt_f64(mean_usize(&group.max_matching_prefixes)),
            fmt_opt_usize(percentile_usize(&group.max_matching_prefixes, 0.50)),
            fmt_opt_f64(mean_usize(&group.matching_states_counts)),
            fmt_opt_f64(mean_usize(&group.strict_path_state_counts)),
            fmt_opt_f64(mean_usize(&group.max_prefix_entries)),
            fmt_opt_f64(mean_f64(&group.mean_matching_prefixes)),
            fmt_opt_f64(mean_usize(&group.beam_sizes)),
            fmt_opt_f64(mean_u64(&group.nodes))
        )?;
    }

    Ok(())
}

fn write_phase_lab_csv(path: &PathBuf, records: &[PhaseLabRecord]) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(
        file,
        "layout,difficulty,scramble_len,iteration,seed,ariadne_solution_len,phase_kind,phase_spec,direct_found,direct_opt_len,direct_elapsed_ms,phase_found,suffix_found,total_found,prefix_len,suffix_len,total_raw_len,total_opt_len,delta_opt_vs_direct,phase_elapsed_ms,suffix_elapsed_ms,total_elapsed_ms,nodes,prefixes_available,prefixes_tested,candidate_min_prefix_len,candidate_prefix_lens,candidate_signatures,reason"
    )?;
    for r in records {
        writeln!(
            file,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            r.layout,
            r.difficulty,
            r.scramble_len,
            r.iteration,
            r.seed,
            r.ariadne_solution_len,
            csv(&r.phase_kind),
            csv(&r.phase_spec),
            r.direct_found,
            r.direct_opt_len,
            r.direct_elapsed_ms,
            r.phase_found,
            r.suffix_found,
            r.total_found,
            r.prefix_len,
            r.suffix_len,
            r.total_raw_len,
            r.total_opt_len,
            r.delta_opt_vs_direct,
            r.phase_elapsed_ms,
            r.suffix_elapsed_ms,
            r.total_elapsed_ms,
            r.nodes,
            r.prefixes_available,
            r.prefixes_tested,
            r.candidate_min_prefix_len,
            csv(&r.candidate_prefix_lens),
            csv(&r.candidate_signatures),
            csv(&r.reason)
        )?;
    }
    Ok(())
}

fn write_phase_lab_markdown(
    path: &PathBuf,
    options: &BenchOptions,
    records: &[PhaseLabRecord],
) -> io::Result<()> {
    let mut groups: BTreeMap<(LayoutId, Difficulty, usize, String, String), PhaseGroupStats> =
        BTreeMap::new();
    for record in records {
        let group = groups
            .entry((
                record.layout,
                record.difficulty,
                record.scramble_len,
                record.phase_kind.clone(),
                record.phase_spec.clone(),
            ))
            .or_default();
        group.total += 1;
        if record.phase_found {
            group.phase_found += 1;
        }
        if record.total_found {
            group.total_found += 1;
        }
        if record.direct_found {
            group.direct_lens.push(record.direct_opt_len);
            let portfolio_len = if record.total_found {
                record.direct_opt_len.min(record.total_opt_len)
            } else {
                record.direct_opt_len
            };
            group.portfolio_lens.push(portfolio_len);
            group
                .portfolio_deltas
                .push(portfolio_len as isize - record.direct_opt_len as isize);
        }
        if record.total_found {
            group.total_lens.push(record.total_opt_len);
            group.deltas.push(record.delta_opt_vs_direct);
            group.prefix_lens.push(record.prefix_len);
            group.suffix_lens.push(record.suffix_len);
            match record.delta_opt_vs_direct.cmp(&0) {
                Ordering::Less => group.better += 1,
                Ordering::Equal => group.equal += 1,
                Ordering::Greater => group.worse += 1,
            }
        }
        group.times.push(record.total_elapsed_ms);
        group.nodes.push(record.nodes);
        group.prefixes_available.push(record.prefixes_available);
        group.prefixes_tested.push(record.prefixes_tested);
        if record.prefixes_available > 0 {
            group
                .candidate_min_prefix_lens
                .push(record.candidate_min_prefix_len);
        }
        *group.reasons.entry(record.reason.clone()).or_insert(0) += 1;
    }

    let mut file = File::create(path)?;
    writeln!(
        file,
        "# {}",
        if options.e_classic_cascade {
            "SkimmIQ E-classic Cascade"
        } else {
            "SkimmIQ Phase Lab"
        }
    )?;
    writeln!(file)?;
    writeln!(file, "- target: `{}`", options.solver.target_mode.label())?;
    writeln!(file, "- layouts: `{}`", join_display(&options.layouts))?;
    writeln!(
        file,
        "- difficulties: `{}`",
        join_display(&options.difficulties)
    )?;
    writeln!(file, "- scrambles: `{:?}`", options.scramble_lengths)?;
    writeln!(
        file,
        "- scramble_profile: `{}`",
        options.scramble_profile.label()
    )?;
    writeln!(file, "- iterations: `{}`", options.iterations)?;
    writeln!(file, "- iteration_start: `{}`", options.iteration_start + 1)?;
    writeln!(file, "- seed: `{}`", options.seed)?;
    if options.e_classic_cascade {
        let near = e_classic_near_pair_options(options);
        let corner = e_classic_corner_options(options);
        writeln!(
            file,
            "- cascade: `direct -> global-near-pair -> auto-corner when unresolved or best>{}`",
            E_CLASSIC_CORNER_QUALITY_THRESHOLD
        )?;
        writeln!(
            file,
            "- near_pair_profile: `miss<=2,prefixes={}/{},rank={},tier={},{},{},phase_ms={},suffix_ms={},stop_gain={}`",
            near.phase_prefixes,
            near.phase_probe_prefixes,
            near.phase_prefix_rank.label(),
            near.phase_tier.max_depth,
            near.phase_tier.width,
            near.phase_tier.restarts,
            near.phase_time_limit_ms,
            near.phase_suffix_time_limit_ms,
            near.phase_stop_after_gain.unwrap_or(0)
        )?;
        writeln!(
            file,
            "- corner_profile: `3-to-9,shielded-depth={},branches={}/{},suffix_batches={}+{},candidates={}/{},band=minimum+{},rank={}({}x{},suffix_probe_ms={},suffix_probe_candidates={}),tier={},{},{},phase_ms={},suffix_ms={}`",
            corner.phase_corner_shielded_body_depth,
            corner.phase_corner_seed_branches,
            corner.phase_corner_arm_branches,
            E_CLASSIC_CORNER_PRIMARY_PREFIXES,
            E_CLASSIC_CORNER_FALLBACK_PREFIXES,
            corner.phase_prefixes,
            corner.phase_probe_prefixes,
            corner.phase_prefix_max_over_min.unwrap_or(0),
            corner.phase_prefix_rank.label(),
            corner.phase_rank_lookahead_depth,
            corner.phase_rank_lookahead_width,
            corner.phase_prefix_suffix_probe_time_limit_ms,
            corner.phase_prefix_suffix_probe_candidates,
            corner.phase_tier.max_depth,
            corner.phase_tier.width,
            corner.phase_tier.restarts,
            corner.phase_time_limit_ms,
            corner.phase_suffix_time_limit_ms
        )?;
    } else {
        writeln!(
            file,
            "- phase_kinds: `{}`",
            join_phase_kinds(&options.phase_kinds)
        )?;
        writeln!(file, "- phase_prefixes: `{}`", options.phase_prefixes)?;
        writeln!(
            file,
            "- phase_prefix_offset: `{}`",
            options.phase_prefix_offset
        )?;
        writeln!(
            file,
            "- phase_probe_prefixes: `{}`",
            options.phase_probe_prefixes
        )?;
        writeln!(
            file,
            "- phase_prefix_max_over_min: `{}`",
            options
                .phase_prefix_max_over_min
                .map(|value| value.to_string())
                .unwrap_or_else(|| "disabled".to_string())
        )?;
        writeln!(
            file,
            "- phase_prefix_rank: `{}`",
            options.phase_prefix_rank.label()
        )?;
        writeln!(
            file,
            "- phase_rank_lookahead: `depth={},width={}`",
            options.phase_rank_lookahead_depth, options.phase_rank_lookahead_width
        )?;
        writeln!(
            file,
            "- phase_prefix_suffix_probe_time_limit_ms: `{}`",
            options.phase_prefix_suffix_probe_time_limit_ms
        )?;
        writeln!(
            file,
            "- phase_prefix_suffix_probe_candidates: `{}`",
            options.phase_prefix_suffix_probe_candidates
        )?;
        writeln!(
            file,
            "- phase_tier: `{},{},{}`",
            options.phase_tier.max_depth, options.phase_tier.width, options.phase_tier.restarts
        )?;
        writeln!(
            file,
            "- phase_time_limit_ms: `{}`",
            options.phase_time_limit_ms
        )?;
        writeln!(
            file,
            "- phase_suffix_time_limit_ms: `{}`",
            options.phase_suffix_time_limit_ms
        )?;
        writeln!(
            file,
            "- phase_profile_portfolio: `{}`",
            options.phase_profile_portfolio
        )?;
        writeln!(file, "- phase_near_misses: `{}`", options.phase_near_misses)?;
        writeln!(
            file,
            "- phase_spec_filter: `{}`",
            if options.phase_spec_filters.is_empty() {
                "none".to_string()
            } else {
                options.phase_spec_filters.join(",")
            }
        )?;
        writeln!(
            file,
            "- phase_direct_threshold: `{}`",
            options.phase_direct_threshold
        )?;
        writeln!(
            file,
            "- phase_stop_after_gain: `{}`",
            options
                .phase_stop_after_gain
                .map(|gain| gain.to_string())
                .unwrap_or_else(|| "disabled".to_string())
        )?;
        writeln!(file, "- phase_skip_direct: `{}`", options.phase_skip_direct)?;
        writeln!(
            file,
            "- phase_corner_shielded: `{}`",
            options.phase_corner_shielded
        )?;
        writeln!(
            file,
            "- phase_corner_shielded_body_depth: `{}`",
            options.phase_corner_shielded_body_depth
        )?;
        writeln!(
            file,
            "- phase_corner_seed_branches: `{}`",
            options.phase_corner_seed_branches
        )?;
        writeln!(
            file,
            "- phase_corner_arm_branches: `{}`",
            options.phase_corner_arm_branches
        )?;
        writeln!(
            file,
            "- phase_corner_pool_specs: `{}`",
            options.phase_corner_pool_specs
        )?;
        writeln!(
            file,
            "- phase_suffix_hard_rescue: `{}`",
            options.phase_suffix_hard_rescue
        )?;
    }
    writeln!(
        file,
        "- operation_profile: `{}`",
        options.solver.operation_profile.label()
    )?;
    writeln!(file, "- beam_rank: `{}`", options.solver.beam_rank.label())?;
    writeln!(file)?;
    writeln!(
        file,
        "| Layout | Difficulty | Scramble | Kind | Spec | Phase Found | Total Found | B/E/W | Mean Direct | Mean Two-Phase | Mean Delta | Mean Portfolio | Portfolio Delta | Mean Prefix | Mean Suffix | Mean Candidate Min | Mean Prefixes T/A | Mean Time ms | Mean Nodes | Reasons |"
    )?;
    writeln!(
        file,
        "|---|---|---:|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|"
    )?;

    for ((layout, difficulty, scramble_len, kind, spec), group) in groups {
        writeln!(
            file,
            "| {} | {} | {} | `{}` | `{}` | {}/{} | {}/{} | {}/{}/{} | {} | {} | {} | {} | {} | {} | {} | {} | {}/{} | {} | {} | `{}` |",
            layout,
            difficulty,
            scramble_len,
            kind,
            spec,
            group.phase_found,
            group.total,
            group.total_found,
            group.total,
            group.better,
            group.equal,
            group.worse,
            fmt_opt_f64(mean_usize(&group.direct_lens)),
            fmt_opt_f64(mean_usize(&group.total_lens)),
            fmt_opt_f64(mean_isize(&group.deltas)),
            fmt_opt_f64(mean_usize(&group.portfolio_lens)),
            fmt_opt_f64(mean_isize(&group.portfolio_deltas)),
            fmt_opt_f64(mean_usize(&group.prefix_lens)),
            fmt_opt_f64(mean_usize(&group.suffix_lens)),
            fmt_opt_f64(mean_usize(&group.candidate_min_prefix_lens)),
            fmt_opt_f64(mean_usize(&group.prefixes_tested)),
            fmt_opt_f64(mean_usize(&group.prefixes_available)),
            fmt_opt_f64(mean_u128(&group.times)),
            fmt_opt_f64(mean_u64(&group.nodes)),
            join_counts(&group.reasons)
        )?;
    }

    Ok(())
}

fn write_markdown(path: &PathBuf, options: &BenchOptions, records: &[RunRecord]) -> io::Result<()> {
    let mut groups: BTreeMap<(LayoutId, Difficulty, usize), GroupStats> = BTreeMap::new();
    for record in records {
        let group = groups
            .entry((record.layout, record.difficulty, record.scramble_len))
            .or_default();
        group.total += 1;
        if record.found {
            group.found += 1;
            group.raw_lens.push(record.raw_solution_len);
            group.opt_lens.push(record.optimized_solution_len);
            group.gains.push(record.gain_vs_scramble);
            group.ratios.push(record.ratio_vs_scramble);
        }
        if record.uniform_solved {
            group.uniform_ok += 1;
        }
        if record.android_solved {
            group.android_ok += 1;
        }
        group.times.push(record.elapsed_ms);
        group.nodes.push(record.nodes);
        if record.first_table_hit {
            group.first_hit_depths.push(record.first_table_hit_depth);
            group
                .first_hit_times
                .push(record.first_table_hit_elapsed_ms);
            group
                .first_hit_suffix_lens
                .push(record.first_table_hit_suffix_len);
        }
        *group
            .methods
            .entry(record.method.label().to_string())
            .or_insert(0) += 1;
        *group
            .target_used
            .entry(record.target_used.clone())
            .or_insert(0) += 1;
        *group
            .operation_profiles
            .entry(record.operation_profile_used.clone())
            .or_insert(0) += 1;
        *group.reasons.entry(record.reason.clone()).or_insert(0) += 1;
    }

    let mut file = File::create(path)?;
    writeln!(file, "# SkimmIQ Ashtree Native Benchmark")?;
    writeln!(file)?;
    writeln!(file, "- target: `{}`", options.solver.target_mode.label())?;
    writeln!(file, "- layouts: `{}`", join_display(&options.layouts))?;
    writeln!(
        file,
        "- difficulties: `{}`",
        join_display(&options.difficulties)
    )?;
    writeln!(file, "- scrambles: `{:?}`", options.scramble_lengths)?;
    writeln!(
        file,
        "- scramble_profile: `{}`",
        options.scramble_profile.label()
    )?;
    writeln!(file, "- iterations: `{}`", options.iterations)?;
    writeln!(file, "- iteration_start: `{}`", options.iteration_start + 1)?;
    writeln!(file, "- seed: `{}`", options.seed)?;
    writeln!(file, "- max_nodes: `{}`", options.solver.max_nodes)?;
    writeln!(file, "- time_limit_ms: `{}`", options.solver.time_limit_ms)?;
    writeln!(
        file,
        "- rescue: `{}` threshold=`{}` time_limit_ms=`{}`",
        options.solver.rescue_enabled,
        options.solver.rescue_threshold,
        options.solver.rescue_time_limit_ms
    )?;
    writeln!(file, "- path_penalty: `{}`", options.solver.path_penalty)?;
    writeln!(file, "- beam_rank: `{}`", options.solver.beam_rank.label())?;
    writeln!(
        file,
        "- corridor_diversity: `{}` prefix_len=`{}` quota_percent=`{}`",
        options.solver.corridor_diversity_enabled,
        options.solver.corridor_prefix_len,
        options.solver.corridor_quota_percent
    )?;
    writeln!(file, "- local_window: `{}`", options.solver.local_window)?;
    writeln!(file, "- local_depth: `{}`", options.solver.local_depth)?;
    writeln!(file, "- hit_patience: `{}`", options.solver.hit_patience)?;
    writeln!(
        file,
        "- hit_restart_patience: `{}`",
        options.solver.hit_restart_patience
    )?;
    writeln!(
        file,
        "- retrograde_suffix_beam_first_hit: `{}`",
        options.solver.retrograde_suffix_beam_first_hit
    )?;
    writeln!(
        file,
        "- portfolio_first_result: `{}`",
        options.solver.portfolio_first_result
    )?;
    writeln!(
        file,
        "- operation_profile: `{}`",
        options.solver.operation_profile.label()
    )?;
    writeln!(
        file,
        "- operation_portfolio: `{}` time_limit_ms=`{}` threshold=`{}`",
        options.solver.operation_portfolio_enabled,
        operation_set_time_limit(&options.solver),
        options.solver.operation_portfolio_threshold
    )?;
    if options.solver.operation_portfolio_enabled
        && options.solver.operation_profile == OperationProfile::Auto
        && options.solver.operation_portfolio_threshold == DEFAULT_OPERATION_PORTFOLIO_THRESHOLD
        && options.layouts.contains(&LayoutId::F)
        && options.difficulties.contains(&Difficulty::Classic)
    {
        writeln!(
            file,
            "- operation_portfolio_effective: `F-classic Auto profiles=basic,pairs,conjugates threshold={} pairs_time_limit_ms={}`",
            DEFAULT_RESCUE_THRESHOLD,
            options.solver.rescue_time_limit_ms
        )?;
    }
    writeln!(
        file,
        "- pattern_db: `{}` depth=`{}` weight=`{}` threshold=`{}`",
        options.solver.pattern_db_enabled,
        options.solver.pattern_db_depth,
        options.solver.pattern_db_weight,
        options.solver.pattern_db_threshold
    )?;
    writeln!(
        file,
        "- f_pattern_db_portfolio: `{}`",
        options.solver.f_pattern_db_portfolio_enabled
    )?;
    writeln!(
        file,
        "- landmark_rescue: `{}` depth=`{}` width=`{}` candidates=`{}` time_limit_ms=`{}` suffix_time_limit_ms=`{}`",
        options.solver.landmark_rescue_enabled,
        options.solver.landmark_depth,
        options.solver.landmark_width,
        options.solver.landmark_candidates,
        options.solver.landmark_time_limit_ms,
        options.solver.landmark_suffix_time_limit_ms
    )?;
    writeln!(
        file,
        "- hard_rescue: `{}` tier=`{},{},{}` time_limit_ms=`{}`",
        options.solver.hard_rescue_enabled,
        options.solver.hard_rescue_tier.max_depth,
        options.solver.hard_rescue_tier.width,
        options.solver.hard_rescue_tier.restarts,
        options.solver.hard_rescue_time_limit_ms
    )?;
    if options.layouts.contains(&LayoutId::E) && options.difficulties.contains(&Difficulty::Classic)
    {
        writeln!(
            file,
            "- e_classic_no_result_hard_rescue_min_tier: `{},{},{}`",
            E_CLASSIC_NO_RESULT_HARD_RESCUE_DEPTH,
            E_CLASSIC_NO_RESULT_HARD_RESCUE_WIDTH,
            E_CLASSIC_NO_RESULT_HARD_RESCUE_RESTARTS
        )?;
    }
    writeln!(
        file,
        "- pair_region_rescue: `{}` table_depth=`{}` forward_depth=`{}` tier=`{},{},{}` prefixes=`{}` time_limit_ms=`{}` suffix_time_limit_ms=`{}` preserve_suffix=`{}`",
        options.solver.pair_region_rescue_enabled,
        options.solver.pair_region_table_depth,
        options.solver.pair_region_forward_depth,
        options.solver.pair_region_tier.max_depth,
        options.solver.pair_region_tier.width,
        options.solver.pair_region_tier.restarts,
        options.solver.pair_region_prefixes,
        options.solver.pair_region_time_limit_ms,
        options.solver.pair_region_suffix_time_limit_ms,
        options.solver.pair_region_preserve_suffix
    )?;
    writeln!(
        file,
        "- region_pair_weight: `{}`",
        options.solver.region_pair_weight
    )?;
    writeln!(file, "- optimize: `{}`", options.solver.optimize)?;
    writeln!(file)?;
    writeln!(
        file,
        "| Layout | Difficulty | Scramble | Found | Android OK | Mean Raw | Mean Opt | P95 Opt | Mean Gain | Mean Ratio | Mean Time ms | P95 Time ms | Mean Nodes | Table Hits | Mean Hit Depth | Mean Hit Time ms | Mean Hit Suffix | Methods | Targets | Profiles | Reasons |"
    )?;
    writeln!(
        file,
        "|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|---|---|---|"
    )?;

    for ((layout, difficulty, scramble_len), group) in groups {
        let mean_raw = mean_usize(&group.raw_lens);
        let mean_opt = mean_usize(&group.opt_lens);
        let p95_opt = percentile_usize(&group.opt_lens, 0.95);
        let mean_gain = mean_isize(&group.gains);
        let mean_ratio = mean_f64(&group.ratios);
        let mean_time = mean_u128(&group.times);
        let p95_time = percentile_u128(&group.times, 0.95);
        let mean_nodes = mean_u64(&group.nodes);
        let table_hits = group.first_hit_depths.len();

        writeln!(
            file,
            "| {} | {} | {} | {}/{} | {}/{} | {} | {} | {} | {} | {} | {} | {} | {} | {}/{} | {} | {} | {} | `{}` | `{}` | `{}` | `{}` |",
            layout,
            difficulty,
            scramble_len,
            group.found,
            group.total,
            group.android_ok,
            group.total,
            fmt_opt_f64(mean_raw),
            fmt_opt_f64(mean_opt),
            fmt_opt_usize(p95_opt),
            fmt_opt_f64(mean_gain),
            fmt_opt_f64(mean_ratio),
            fmt_opt_f64(mean_time),
            fmt_opt_u128(p95_time),
            fmt_opt_f64(mean_nodes),
            table_hits,
            group.total,
            fmt_opt_f64(mean_usize(&group.first_hit_depths)),
            fmt_opt_f64(mean_u128(&group.first_hit_times)),
            fmt_opt_f64(mean_usize(&group.first_hit_suffix_lens)),
            join_counts(&group.methods),
            join_counts(&group.target_used),
            join_counts(&group.operation_profiles),
            join_counts(&group.reasons)
        )?;
    }

    Ok(())
}

fn csv(value: &str) -> String {
    let escaped = value.replace('"', "\"\"");
    format!("\"{escaped}\"")
}

fn join_display<T: fmt::Display>(values: &[T]) -> String {
    values
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn join_usize(values: &[usize]) -> String {
    values
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(";")
}

fn join_usize_counts(values: &[usize]) -> String {
    values
        .iter()
        .enumerate()
        .map(|(index, value)| format!("{index}:{value}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn join_f64(values: &[f64]) -> String {
    values
        .iter()
        .map(|value| format!("{value:.3}"))
        .collect::<Vec<_>>()
        .join(";")
}

fn join_phase_kinds(values: &[PhaseKind]) -> String {
    values
        .iter()
        .map(|kind| kind.label().to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn join_projection_kinds(values: &[ProjectionKind]) -> String {
    values
        .iter()
        .map(|kind| kind.label().to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn join_counts(counts: &BTreeMap<String, usize>) -> String {
    counts
        .iter()
        .map(|(key, value)| format!("{key}:{value}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn puzzle_sequence_text_for_record(
    layout: LayoutId,
    difficulty: Difficulty,
    moves: &[MoveIndex],
) -> String {
    Puzzle::new(layout, difficulty).moves_text(moves)
}

fn mean_usize(values: &[usize]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    Some(values.iter().sum::<usize>() as f64 / values.len() as f64)
}

fn mean_isize(values: &[isize]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    Some(values.iter().sum::<isize>() as f64 / values.len() as f64)
}

fn mean_i32(values: &[i32]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    Some(values.iter().sum::<i32>() as f64 / values.len() as f64)
}

fn mean_f64(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    Some(values.iter().sum::<f64>() / values.len() as f64)
}

fn mean_u128(values: &[u128]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    Some(values.iter().sum::<u128>() as f64 / values.len() as f64)
}

fn mean_u64(values: &[u64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    Some(values.iter().sum::<u64>() as f64 / values.len() as f64)
}

fn percentile_usize(values: &[usize], p: f64) -> Option<usize> {
    percentile(values, p).copied()
}

fn percentile_u128(values: &[u128], p: f64) -> Option<u128> {
    percentile(values, p).copied()
}

fn percentile_f64(values: &[f64], p: f64) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));
    let index = ((sorted.len() - 1) as f64 * p).round() as usize;
    sorted.get(index).copied()
}

fn percentile<T: Ord>(values: &[T], p: f64) -> Option<&T> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.iter().collect::<Vec<_>>();
    sorted.sort_unstable();
    let idx = ((sorted.len() - 1) as f64 * p).ceil() as usize;
    sorted.get(idx).copied()
}

fn fmt_opt_f64(value: Option<f64>) -> String {
    value
        .map(|v| format!("{v:.3}"))
        .unwrap_or_else(|| "-".to_string())
}

fn fmt_opt_usize(value: Option<usize>) -> String {
    value
        .map(|v| v.to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn fmt_opt_i32(value: Option<i32>) -> String {
    value
        .map(|v| v.to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn fmt_opt_u8(value: Option<u8>) -> String {
    value
        .map(|v| v.to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn fmt_opt_u128(value: Option<u128>) -> String {
    value
        .map(|v| v.to_string())
        .unwrap_or_else(|| "-".to_string())
}

impl PartialEq for Move {
    fn eq(&self, other: &Self) -> bool {
        self.axis == other.axis && self.layer == other.layer && self.direction == other.direction
    }
}

impl Eq for Move {}

impl PartialOrd for Move {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Move {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.axis, self.layer, self.direction).cmp(&(other.axis, other.layer, other.direction))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solved_e_classic_has_nine_android_solved_tapes() {
        let puzzle = Puzzle::new(LayoutId::E, Difficulty::Classic);
        let tapes = puzzle_tape_coords(&puzzle);
        assert_eq!(tapes.len(), 9);
        assert!(tapes.iter().copied().all(|tape| {
            tapes_are_jointly_android_solved(&puzzle, &puzzle.solved_colors, &[tape])
        }));
    }

    #[test]
    fn e_classic_builds_all_cross_axis_tape_pairs() {
        let puzzle = Puzzle::new(LayoutId::E, Difficulty::Classic);
        let specs = build_phase_specs(&puzzle, &[PhaseKind::CrossAxisTapePair], 0);
        assert_eq!(specs.len(), 27);
        assert!(specs
            .iter()
            .all(|spec| spec.tapes.len() == 2 && spec.tapes[0].axis != spec.tapes[1].axis));
    }

    #[test]
    fn e_classic_builds_all_three_axis_tape_triplets() {
        let puzzle = Puzzle::new(LayoutId::E, Difficulty::Classic);
        let specs = build_phase_specs(&puzzle, &[PhaseKind::ThreeAxisTapeTriplet], 0);
        assert_eq!(specs.len(), 27);
        assert!(specs.iter().all(|spec| {
            spec.tapes.len() == 3
                && spec.tapes[0].axis == 0
                && spec.tapes[1].axis == 1
                && spec.tapes[2].axis == 2
        }));
    }

    #[test]
    fn e_classic_builds_binary_color_splits() {
        let puzzle = Puzzle::new(LayoutId::E, Difficulty::Classic);
        let specs = build_phase_specs(&puzzle, &[PhaseKind::BinaryColorSplit], 0);

        assert_eq!(specs.len(), 10);
        assert!(specs.iter().all(|spec| spec.faces.len() == 3));
        assert!(specs
            .iter()
            .all(|spec| binary_split_color_sets(&puzzle, &spec.faces).is_some()));
    }

    #[test]
    fn e_classic_builds_layer_band_specs() {
        let puzzle = Puzzle::new(LayoutId::E, Difficulty::Classic);
        let strict = build_phase_specs(&puzzle, &[PhaseKind::OppositeLayerBand], 0);
        let class = build_phase_specs(&puzzle, &[PhaseKind::OppositeLayerClassBand], 0);

        assert_eq!(strict.len(), 3);
        assert_eq!(class.len(), 3);
        assert!(strict.iter().all(|spec| spec.faces.len() == 2));
        assert!(class.iter().all(|spec| spec.faces.len() == 2));
    }

    #[test]
    fn e_classic_macro_shift_preservation_matches_ring_geometry() {
        let puzzle = Puzzle::new(LayoutId::E, Difficulty::Classic);

        for shift in [1, 2] {
            assert!((0..puzzle.moves.len()).any(|move_index| {
                let (axis_ok, parity_ok) = macro_shift_preserves(&puzzle, move_index, shift);
                !axis_ok && !parity_ok
            }));
        }

        assert!((0..puzzle.moves.len()).all(|move_index| {
            let (axis_ok, parity_ok) = macro_shift_preserves(&puzzle, move_index, 3);
            !axis_ok && parity_ok
        }));
        assert!((0..puzzle.moves.len()).all(|move_index| {
            let (axis_ok, parity_ok) = macro_shift_preserves(&puzzle, move_index, 6);
            axis_ok && parity_ok
        }));
    }

    #[test]
    fn e_classic_macro6_target_suffix_solves_table_states() {
        let puzzle = Puzzle::new(LayoutId::E, Difficulty::Classic);
        let config = parse_bench_options(vec![
            "--max-nodes".to_string(),
            "1000000".to_string(),
            "--time-limit-ms".to_string(),
            "30000".to_string(),
        ])
        .expect("options")
        .solver;
        let artifacts =
            build_macro_target_artifacts(&puzzle, TargetMode::Android, 6, 3, 0, &config)
                .expect("macro target artifacts");

        assert_eq!(artifacts.macro_ops.len(), 9);
        assert!(artifacts.table.len() > 1);

        for colors in artifacts.target_colors.iter().take(50) {
            let suffix = *artifacts
                .table
                .get(&state_key(colors))
                .expect("table suffix");
            let moves = expand_macro_suffix(&artifacts.macro_ops, suffix);
            let mut solved = colors.clone();
            puzzle.apply_moves(&mut solved, &moves);
            assert!(is_android_solved(
                &solved,
                &puzzle.face_indexes,
                puzzle.difficulty
            ));
        }

        let mut shifted = puzzle.solved_colors.clone();
        let first_macro = &artifacts.macro_ops[0];
        apply_macro_to_colors(
            &puzzle,
            &mut shifted,
            first_macro.move_index,
            first_macro.shift,
        );
        let suffix = *artifacts
            .table
            .get(&state_key(&shifted))
            .expect("shifted suffix");
        assert_eq!(suffix.len, 1);
    }

    #[test]
    fn e_classic_axis_ring_target_suffix_solves_table_states() {
        let puzzle = Puzzle::new(LayoutId::E, Difficulty::Classic);
        let config = parse_bench_options(vec![
            "--max-nodes".to_string(),
            "1000000".to_string(),
            "--time-limit-ms".to_string(),
            "30000".to_string(),
        ])
        .expect("options")
        .solver;

        for axis in 0..3_u8 {
            let artifacts = build_restricted_target_artifacts(
                &puzzle,
                TargetMode::AndroidMultiGoal,
                axis_moves(&puzzle, axis),
                8,
                0,
                &config,
            )
            .expect("axis ring artifacts");

            assert_eq!(artifacts.allowed_moves.len(), 6);
            assert_eq!(artifacts.table.len(), 20_736);
            assert_eq!(artifacts.table.values().map(Vec::len).max(), Some(8));

            for colors in artifacts.target_colors.iter().take(50) {
                assert_eq!(
                    axis_ring_order_score(&puzzle, colors, axis, &artifacts.axis_ring_profiles),
                    0
                );
                let suffix = artifacts
                    .table
                    .get(&state_key(colors))
                    .expect("axis ring suffix");
                let mut solved = colors.clone();
                puzzle.apply_moves(&mut solved, suffix);
                assert!(is_android_solved(
                    &solved,
                    &puzzle.face_indexes,
                    puzzle.difficulty
                ));
            }
        }
    }

    #[test]
    fn restricted_target_expansion_suffix_solves_one_step_shell() {
        let puzzle = Puzzle::new(LayoutId::E, Difficulty::Classic);
        let axis = 0_u8;
        let config = parse_bench_options(vec![
            "--target-expand-depth".to_string(),
            "1".to_string(),
            "--max-nodes".to_string(),
            "10000000".to_string(),
            "--time-limit-ms".to_string(),
            "30000".to_string(),
        ])
        .expect("options")
        .solver;
        let artifacts = build_restricted_target_artifacts(
            &puzzle,
            TargetMode::Android,
            axis_moves(&puzzle, axis),
            18,
            0,
            &config,
        )
        .expect("expanded axis ring artifacts");

        assert!(artifacts.table.len() > 1_728);
        assert!(
            artifacts
                .table
                .values()
                .map(Vec::len)
                .max()
                .unwrap_or_default()
                <= 19
        );

        let non_axis_move = (0..puzzle.moves.len())
            .find(|&move_index| puzzle.moves[move_index].axis != axis)
            .expect("non-axis move");
        let mut shifted = puzzle.solved_colors.clone();
        puzzle.apply_move(&mut shifted, non_axis_move);
        let suffix = artifacts
            .table
            .get(&state_key(&shifted))
            .expect("one-step expanded suffix");
        assert_eq!(suffix.len(), 1);

        puzzle.apply_moves(&mut shifted, suffix);
        assert!(is_android_solved(
            &shifted,
            &puzzle.face_indexes,
            puzzle.difficulty
        ));
    }

    #[test]
    fn cyclic_hamming_accepts_ring_rotations() {
        let target = vec![0, 0, 1, 1, 2, 2, 3, 3];
        let rotated = vec![2, 3, 3, 0, 0, 1, 1, 2];
        assert_eq!(cyclic_hamming_misses(&rotated, &target), 0);

        let one_wrong = vec![2, 3, 3, 0, 0, 1, 1, 4];
        assert_eq!(cyclic_hamming_misses(&one_wrong, &target), 1);
    }

    #[test]
    fn min_cost_assignment_dp_finds_exact_matching() {
        let costs = vec![vec![9, 1, 8], vec![2, 8, 9], vec![7, 9, 3]];
        let (cost, assignment) = min_cost_assignment_dp(&costs);
        assert_eq!(cost, 6);
        assert_eq!(assignment, vec![1, 0, 2]);
    }

    #[test]
    fn permutation_cycle_cost_counts_nontrivial_cycles() {
        assert_eq!(permutation_cycle_cost(&[1, 2, 0, 3]), 2);
        assert_eq!(permutation_cycle_cost(&[1, 0, 3, 2]), 2);
        assert_eq!(permutation_cycle_cost(&[0, 1, 2, 3]), 0);
    }

    #[test]
    fn commutator_cycle_type_prioritizes_clean_primitives() {
        assert_eq!(commutator_cycle_type(&[3]), "clean-3-cycle");
        assert_eq!(commutator_cycle_type(&[2, 2]), "double-transposition");
        assert_eq!(commutator_cycle_type(&[2, 3]), "2+3");
    }

    #[test]
    fn e_classic_corner_plans_grow_three_stickers_to_twelve() {
        let puzzle = Puzzle::new(LayoutId::E, Difficulty::Classic);
        let specs = build_phase_specs(&puzzle, &[PhaseKind::ProtectedCornerBlock], 0);
        assert_eq!(specs.len(), 8);
        for spec in specs {
            let plan = spec.corner_plan.expect("corner plan");
            assert_eq!(plan.seed_regions.len(), 3);
            assert_eq!(
                plan.seed_regions
                    .iter()
                    .map(|(_, indexes)| indexes.len())
                    .sum::<usize>(),
                3
            );
            assert_eq!(
                plan.arm_regions
                    .iter()
                    .map(|(_, indexes)| indexes.len())
                    .sum::<usize>(),
                9
            );
            assert_eq!(
                plan.block_regions
                    .iter()
                    .map(|(_, indexes)| indexes.len())
                    .sum::<usize>(),
                12
            );
            assert_eq!(plan.forbidden_tapes.len(), 3);
            assert!(corner_phase_target_solved(
                &puzzle,
                &puzzle.solved_colors,
                &plan,
                0
            ));
        }
    }

    #[test]
    fn corner_target_does_not_require_three_whole_faces() {
        let puzzle = Puzzle::new(LayoutId::E, Difficulty::Classic);
        let spec = build_phase_specs(&puzzle, &[PhaseKind::ProtectedCornerBlock], 0)
            .into_iter()
            .next()
            .expect("corner spec");
        let plan = spec.corner_plan.as_ref().expect("corner plan");
        let mut colors = puzzle.solved_colors.clone();
        for &face in &spec.faces {
            let block = plan
                .block_regions
                .iter()
                .find(|(region_face, _)| *region_face == face)
                .map(|(_, indexes)| indexes)
                .expect("face block");
            let outside = puzzle.face_indexes[face.index()]
                .iter()
                .copied()
                .find(|index| !block.contains(index))
                .expect("outside sticker");
            colors[outside] = (colors[outside] + 1) % 6;
        }
        assert!(spec.faces.iter().copied().all(|face| {
            analyze_face(&puzzle, &colors, face).is_some_and(|analysis| analysis.misses > 0)
        }));
        assert!(phase_target_solved(&puzzle, &colors, &spec));
    }

    #[test]
    fn shielded_corner_operations_restore_all_three_stickers() {
        let puzzle = Puzzle::new(LayoutId::E, Difficulty::Classic);
        let spec = build_phase_specs(&puzzle, &[PhaseKind::ProtectedCornerBlock], 0)
            .into_iter()
            .next()
            .expect("corner spec");
        let plan = spec.corner_plan.as_ref().expect("corner plan");
        let base = build_operations(&puzzle, OperationProfile::ExpandedParallel);
        let strict = build_corner_stage_operations(&puzzle, plan, &base, false, 1);
        let shielded = build_corner_stage_operations(&puzzle, plan, &base, true, 2);
        assert!(shielded.len() > strict.len());

        let seed_indexes = plan
            .seed_regions
            .iter()
            .flat_map(|(_, indexes)| indexes.iter().copied())
            .collect::<Vec<_>>();
        for operation in shielded {
            let permutation = operation_permutation_key(&puzzle, &operation.moves);
            assert!(seed_indexes
                .iter()
                .all(|&index| permutation[index] as usize == index));
        }
    }

    #[test]
    fn repeated_tape_moves_compress_to_the_shortest_direction() {
        let puzzle = Puzzle::new(LayoutId::E, Difficulty::Classic);
        let plus = puzzle.find_move(0, 0, 1).expect("x0+");
        let compressed = compress_same_tape_runs(&puzzle, &vec![plus; 7]);
        assert_eq!(compressed.len(), 5);
        assert!(compressed
            .iter()
            .all(|&move_index| puzzle.moves[move_index].direction == -1));
    }

    #[test]
    fn ariadne_reduces_same_tape_moves_across_commuting_layers() {
        let puzzle = Puzzle::new(LayoutId::D, Difficulty::Classic);
        let x0_plus = puzzle.find_move(0, 0, 1).expect("x0+");
        let x0_minus = puzzle.find_move(0, 0, -1).expect("x0-");
        let x1_plus = puzzle.find_move(0, 1, 1).expect("x1+");
        let y0_plus = puzzle.find_move(1, 0, 1).expect("y0+");

        assert_eq!(ariadne_solution_len(&puzzle, &[x0_plus, x0_plus]), 1);
        assert_eq!(
            ariadne_solution_len(&puzzle, &[x0_plus, x1_plus, x0_minus]),
            1
        );
        assert_eq!(
            ariadne_solution_len(&puzzle, &[x0_plus, y0_plus, x0_minus]),
            3
        );
    }

    #[test]
    fn ariadne_solution_moves_expand_and_solve_reduced_history() {
        let puzzle = Puzzle::new(LayoutId::D, Difficulty::Classic);
        let x0_plus = puzzle.find_move(0, 0, 1).expect("x0+");
        let history = vec![x0_plus, x0_plus];

        assert_eq!(ariadne_solution_len(&puzzle, &history), 1);
        let solution = ariadne_solution_moves(&puzzle, &history);
        assert_eq!(solution.len(), 2);

        let mut colors = puzzle.solved_colors.clone();
        puzzle.apply_moves(&mut colors, &history);
        puzzle.apply_moves(&mut colors, &solution);
        assert_eq!(colors, puzzle.solved_colors);
    }

    #[test]
    fn ariadne_solution_moves_solve_android_original_scramble() {
        let puzzle = Puzzle::new(LayoutId::E, Difficulty::Classic);
        let scramble = android_original_scramble(&puzzle, 20, 1592598566);
        let solution = ariadne_solution_moves(&puzzle, &scramble);

        let mut colors = puzzle.solved_colors.clone();
        puzzle.apply_moves(&mut colors, &scramble);
        puzzle.apply_moves(&mut colors, &solution);
        assert_eq!(colors, puzzle.solved_colors);
        assert!(is_android_solved(
            &colors,
            &puzzle.face_indexes,
            Difficulty::Classic
        ));
    }

    #[test]
    fn android_original_scramble_avoids_repeating_an_axis() {
        let puzzle = Puzzle::new(LayoutId::E, Difficulty::Moderate);
        let scramble = android_original_scramble(&puzzle, 100, 12345);

        assert_eq!(scramble.len(), 100);
        assert!(scramble
            .windows(2)
            .all(|pair| { puzzle.moves[pair[0]].axis != puzzle.moves[pair[1]].axis }));
    }

    #[test]
    fn android_easy_scramble_prefers_non_uniform_tapes_when_available() {
        let puzzle = Puzzle::new(LayoutId::E, Difficulty::Easy);
        let scramble = android_original_scramble(&puzzle, 100, 54321);
        let dims = puzzle.layout.dims();
        let mut last_axis = None;

        for move_index in scramble {
            let mv = &puzzle.moves[move_index];
            let axis_pool = [0_u8, 1, 2]
                .into_iter()
                .filter(|axis| Some(*axis) != last_axis)
                .collect::<Vec<_>>();
            let has_non_uniform = axis_pool.iter().any(|axis| {
                let layer_count = match axis {
                    0 => dims.rows,
                    1 => dims.cols,
                    _ => dims.layers,
                };
                (0..layer_count)
                    .any(|layer| !tape_is_uniform(&puzzle, *axis, layer, &puzzle.solved_colors))
            });
            if has_non_uniform {
                assert!(!tape_is_uniform(
                    &puzzle,
                    mv.axis,
                    mv.layer,
                    &puzzle.solved_colors
                ));
            }
            last_axis = Some(mv.axis);
        }
    }

    #[test]
    fn prefix_lookahead_keeps_the_current_state_as_a_candidate() {
        let puzzle = Puzzle::new(LayoutId::E, Difficulty::Classic);
        let mut colors = puzzle.solved_colors.clone();
        let scramble = [
            puzzle.find_move(0, 0, 1).expect("x0+"),
            puzzle.find_move(1, 1, -1).expect("y1-"),
            puzzle.find_move(2, 2, 1).expect("z2+"),
        ];
        puzzle.apply_moves(&mut colors, &scramble);
        let current = score_state(&colors, &puzzle, TargetMode::Android, 0, None);
        let lookahead =
            phase_prefix_lookahead_score(&puzzle, &colors, TargetMode::Android, 0, None, 3, 64);
        assert!(lookahead <= current);
    }

    #[test]
    fn corridor_diverse_selection_reserves_unique_prefixes() {
        let entry = |path: Vec<MoveIndex>, rank_score: i32| BeamEntry {
            colors: Vec::new(),
            path,
            last_move: None,
            score: rank_score,
            rank_score,
        };
        let candidates = vec![
            entry(vec![1, 2], 1),
            entry(vec![1, 2, 3], 2),
            entry(vec![1, 2, 4], 3),
            entry(vec![3, 4], 100),
            entry(vec![5, 6], 101),
        ];

        let classic = select_beam_survivors(candidates.clone(), 3, false, 2, 100);
        assert_eq!(
            classic
                .iter()
                .map(|entry| entry.path.clone())
                .collect::<Vec<_>>(),
            vec![vec![1, 2], vec![1, 2, 3], vec![1, 2, 4]]
        );

        let diverse = select_beam_survivors(candidates, 3, true, 2, 100);
        assert_eq!(
            diverse
                .iter()
                .map(|entry| entry.path.clone())
                .collect::<Vec<_>>(),
            vec![vec![1, 2], vec![3, 4], vec![5, 6]]
        );
    }

    #[test]
    fn pooled_corner_prefixes_keep_the_shortest_path_to_each_state() {
        let puzzle = Puzzle::new(LayoutId::E, Difficulty::Classic);
        let x = puzzle.find_move(0, 0, 1).expect("x0+");
        let y = puzzle.find_move(1, 1, 1).expect("y1+");
        let y_inverse = puzzle.inverse_index(y);
        let short = vec![x];
        let long = vec![x, y, y_inverse];
        let mut prefixes = vec![long, short.clone()];

        dedupe_phase_prefix_states(&puzzle, &puzzle.solved_colors, &mut prefixes);

        assert_eq!(prefixes, vec![short]);
    }

    #[test]
    fn e_classic_cascade_uses_fixed_stages_and_rescue_gate() {
        let options =
            parse_e_classic_cascade_options(vec!["--iterations".to_string(), "1".to_string()])
                .expect("cascade options");
        assert_eq!(options.layouts, vec![LayoutId::E]);
        assert_eq!(options.difficulties, vec![Difficulty::Classic]);
        assert_eq!(options.scramble_lengths, vec![80]);
        assert_eq!(options.solver.target_mode, TargetMode::AndroidPortfolio);
        assert_eq!(
            options.phase_kinds,
            vec![
                PhaseKind::AllOppositeAndroidNearPairs,
                PhaseKind::ProtectedCornerArms
            ]
        );
        assert!(options.e_classic_cascade);
        assert_eq!(
            e_classic_corner_options(&options).phase_prefixes,
            E_CLASSIC_CORNER_PRIMARY_PREFIXES + E_CLASSIC_CORNER_FALLBACK_PREFIXES
        );

        assert!(!should_run_e_classic_corner_rescue(19, false, 0, false, 0));
        assert!(should_run_e_classic_corner_rescue(20, false, 0, false, 0));
        assert!(!should_run_e_classic_corner_rescue(20, false, 0, true, 74));
        assert!(!should_run_e_classic_corner_rescue(20, true, 80, true, 80));
        assert!(should_run_e_classic_corner_rescue(20, true, 81, true, 74));
        assert!(should_run_e_classic_corner_rescue(20, true, 70, true, 81));

        let axis_options = parse_e_classic_cascade_options(vec![
            "--axis-ring-rescue".to_string(),
            "--axis-ring-rescue-threshold".to_string(),
            "60".to_string(),
        ])
        .expect("cascade options with axis-ring rescue");
        let puzzle = Puzzle::new(LayoutId::E, Difficulty::Classic);
        assert!(axis_options.axis_ring_rescue_enabled);
        assert_eq!(axis_options.axis_ring_rescue_threshold, 60);
        assert_eq!(axis_options.axis_ring_rescue_expand_depth, 1);
        assert_eq!(axis_options.axis_ring_rescue_tier.max_depth, 70);
        assert_eq!(axis_options.axis_ring_rescue_tier.width, 3000);
        assert_eq!(axis_options.axis_ring_rescue_tier.restarts, 3);
        assert_eq!(
            axis_options.axis_ring_rescue_position,
            AxisRingRescuePosition::AfterCascade
        );
        assert_eq!(axis_options.axis_ring_rescue_corner_skip_threshold, 65);
        assert!(!should_run_axis_ring_rescue(
            &axis_options,
            &puzzle,
            true,
            59
        ));
        assert!(should_run_axis_ring_rescue(
            &axis_options,
            &puzzle,
            true,
            60
        ));
        assert!(should_run_axis_ring_rescue(
            &axis_options,
            &puzzle,
            false,
            0
        ));

        let axis_before_corner = parse_e_classic_cascade_options(vec![
            "--axis-ring-rescue".to_string(),
            "--axis-ring-rescue-position".to_string(),
            "before-corner".to_string(),
            "--axis-ring-rescue-corner-skip-threshold".to_string(),
            "70".to_string(),
        ])
        .expect("cascade options with axis-ring before corner");
        assert_eq!(
            axis_before_corner.axis_ring_rescue_position,
            AxisRingRescuePosition::BeforeCorner
        );
        assert_eq!(
            axis_before_corner.axis_ring_rescue_corner_skip_threshold,
            70
        );
        assert!(should_skip_corner_after_axis_ring(&axis_before_corner, 70));
        assert!(!should_skip_corner_after_axis_ring(&axis_before_corner, 71));

        let skip_direct = parse_e_classic_cascade_options(vec!["--phase-skip-direct".to_string()])
            .expect("cascade options with direct disabled");
        assert!(skip_direct.phase_skip_direct);

        let android_scramble = parse_bench_options(vec![
            "--scramble-profile".to_string(),
            "android-original".to_string(),
        ])
        .expect("Android scramble profile");
        assert_eq!(
            android_scramble.scramble_profile,
            ScrambleProfile::AndroidOriginal
        );
    }
}
