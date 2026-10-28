//! The assistant's command surface for the OneOS host bridge (`craft:command {method: "command.batch"}`).
//!
//! The host relays an allow-listed run: `{label, steps: [{name, args?}]}`. The whole run is judged BEFORE anything runs (an id
//! outside the list, an argument that names a file or an address, a private `__` argument, or a step reference to a step that
//! has not run refuses the run), every step runs through the same registry the menus, the control channel and the MCP server
//! use (so a step the document's state forbids is refused exactly as it would be for the person), and the run is ONE history
//! step named by `label`: a step that fails takes back every step before it, so a run is all or nothing. There is no
//! file-path or dialog command in the list: files move only through the host.
//!
//! A later step may name the layer an earlier step made: `{"$step": n}` stands for the `id` (else the `layer`) of step n's
//! result, `{"$step": n, "field": "..."}` for another field of it.
//!
//! This list is the one declared in the host's `platform-craft` registry as `photo` commands (pinned equal there).

use serde_json::{Value, json};

use crate::Session;

/// Registry commands the assistant may run. Each acts at once on the document (no dialog, no file picker, no pointer gesture),
/// and none is Undo or Redo: undoing a run is the person's.
pub const ASSISTANT_COMMANDS: &[&str] = &[
    // edit.*
    "edit.fill",
    "edit.clear",
    // select.*
    "select.all",
    "select.deselect",
    "select.inverse",
    "select.rect",
    // layer.*
    "layer.new.layer",
    "layer.new.group",
    "layer.groupLayers",
    "layer.duplicate",
    "layer.delete",
    "layer.select",
    "layer.setProps",
    "layer.arrange.bringForward",
    "layer.arrange.sendBackward",
    "layer.arrange.bringToFront",
    "layer.arrange.sendToBack",
    "layer.createClippingMask",
    "layer.releaseClippingMask",
    "layer.layerMask.revealAll",
    "layer.layerMask.hideAll",
    "layer.layerMask.revealSelection",
    "layer.layerMask.delete",
    "layer.mergeDown",
    "layer.mergeGroup",
    "layer.flattenImage",
    "layer.newFillLayer.solidColor",
    "layer.newFillLayer.gradient",
    "layer.setAdjustment",
    "layer.moveTo",
    "layer.translate",
    // image.*
    "image.imageRotation.flipCanvasHorizontal",
    "image.imageRotation.flipCanvasVertical",
    "image.imageRotation.180",
    "image.imageRotation.90cw",
    "image.imageRotation.90ccw",
    "image.adjustments.desaturate",
    // paint.*
    "paint.stroke",
    // tools.*
    "tools.setColors",
    "tools.swapColors",
    "tools.defaultColors",
    // session.*
    "session.inspect",
    // document.*
    "document.inspect",
    // command.*
    "command.list",
    // document.*
    "document.pixel",
    // layer.*
    "layer.newAdjustmentLayer.brightnessContrast",
    // image.*
    "image.adjustments.brightnessContrast",
    // layer.*
    "layer.newAdjustmentLayer.levels",
    // image.*
    "image.adjustments.levels",
    // layer.*
    "layer.newAdjustmentLayer.curves",
    // image.*
    "image.adjustments.curves",
    // layer.*
    "layer.newAdjustmentLayer.exposure",
    // image.*
    "image.adjustments.exposure",
    // layer.*
    "layer.newAdjustmentLayer.vibrance",
    // image.*
    "image.adjustments.vibrance",
    // layer.*
    "layer.newAdjustmentLayer.hueSaturation",
    // image.*
    "image.adjustments.hueSaturation",
    // layer.*
    "layer.newAdjustmentLayer.colorBalance",
    // image.*
    "image.adjustments.colorBalance",
    // layer.*
    "layer.newAdjustmentLayer.blackWhite",
    // image.*
    "image.adjustments.blackWhite",
    // layer.*
    "layer.newAdjustmentLayer.photoFilter",
    // image.*
    "image.adjustments.photoFilter",
    // layer.*
    "layer.newAdjustmentLayer.channelMixer",
    // image.*
    "image.adjustments.channelMixer",
    // layer.*
    "layer.newAdjustmentLayer.invert",
    // image.*
    "image.adjustments.invert",
    // layer.*
    "layer.newAdjustmentLayer.posterize",
    // image.*
    "image.adjustments.posterize",
    // layer.*
    "layer.newAdjustmentLayer.threshold",
    // image.*
    "image.adjustments.threshold",
    // layer.*
    "layer.newAdjustmentLayer.gradientMap",
    // image.*
    "image.adjustments.gradientMap",
    // layer.*
    "layer.newAdjustmentLayer.selectiveColor",
    // image.*
    "image.adjustments.selectiveColor",
    // layer.*
    "layer.layerStyle.dropShadow",
    "layer.layerStyle.innerShadow",
    "layer.layerStyle.outerGlow",
    "layer.layerStyle.innerGlow",
    "layer.layerStyle.stroke",
    "layer.layerStyle.colorOverlay",
    "layer.layerStyle.gradientOverlay",
    "layer.layerStyle.patternOverlay",
    "layer.layerStyle.bevelEmboss",
    "layer.layerStyle.satin",
    "layer.layerStyle.replace",
    "layer.layerStyle.makeDefault",
    "layer.layerStyle.defaultFor",
    "layer.layerStyle.clear",
    "layer.layerStyle.contours.list",
    "layer.layerStyle.contours.new",
    "layer.layerStyle.contours.delete",
    "layer.list",
    // filter.*
    "filter.blur.gaussianBlur",
    "filter.blur.blur",
    "filter.blur.blurMore",
    "filter.sharpen.sharpen",
    "filter.sharpen.sharpenMore",
    "filter.sharpen.sharpenEdges",
    "filter.noise.despeckle",
    "filter.blur.boxBlur",
    "filter.blur.motionBlur",
    "filter.blur.radialBlur",
    "filter.blur.surfaceBlur",
    "filter.sharpen.unsharpMask",
    "filter.sharpen.smartSharpen",
    "filter.noise.addNoise",
    "filter.noise.median",
    "filter.noise.dustAndScratches",
    "filter.pixelate.mosaic",
    "filter.stylize.emboss",
    "filter.stylize.findEdges",
    "filter.stylize.solarize",
    "filter.distort.twirl",
    "filter.distort.pinch",
    "filter.distort.spherize",
    "filter.distort.wave",
    "filter.distort.ripple",
    "filter.distort.polarCoordinates",
    "filter.other.highPass",
    "filter.other.minimum",
    "filter.other.maximum",
    "filter.other.offset",
    "filter.lastFilter",
    "filter.pixelate.colorHalftone",
    "filter.pixelate.crystallize",
    "filter.pixelate.facet",
    "filter.pixelate.fragment",
    "filter.pixelate.mezzotint",
    "filter.pixelate.pointillize",
    "filter.stylize.diffuse",
    "filter.stylize.extrude",
    "filter.stylize.oilPaint",
    "filter.stylize.tiles",
    "filter.stylize.traceContour",
    "filter.stylize.wind",
    "filter.distort.shear",
    "filter.distort.zigZag",
    "filter.render.clouds",
    "filter.render.differenceClouds",
    "filter.render.fibers",
    "filter.render.lensFlare",
    "filter.render.lightingEffects",
    "filter.render.relight",
    "filter.noise.reduceNoise",
    "filter.blur.smartBlur",
    "filter.blur.lensBlur",
    "filter.blur.shapeBlur",
    "filter.blurGallery.tiltShift",
    "filter.blurGallery.irisBlur",
    "filter.blurGallery.fieldBlur",
    "filter.blurGallery.spinBlur",
    "filter.blurGallery.pathBlur",
    "filter.other.custom",
    "filter.other.hsbHsl",
    "filter.video.deInterlace",
    "filter.video.ntscColors",
    "filter.other.colorToAlpha",
    "filter.filterGallery",
    "filter.gallery.coloredPencil",
    "filter.gallery.cutout",
    "filter.gallery.dryBrush",
    "filter.gallery.filmGrain",
    "filter.gallery.fresco",
    "filter.gallery.neonGlow",
    "filter.gallery.paintDaubs",
    "filter.gallery.paletteKnife",
    "filter.gallery.plasticWrap",
    "filter.gallery.posterEdges",
    "filter.gallery.roughPastels",
    "filter.gallery.smudgeStick",
    "filter.gallery.sponge",
    "filter.gallery.underpainting",
    "filter.gallery.watercolor",
    "filter.gallery.accentedEdges",
    "filter.gallery.angledStrokes",
    "filter.gallery.crosshatch",
    "filter.gallery.darkStrokes",
    "filter.gallery.inkOutlines",
    "filter.gallery.spatter",
    "filter.gallery.sprayedStrokes",
    "filter.gallery.sumiE",
    "filter.gallery.diffuseGlow",
    "filter.gallery.glass",
    "filter.gallery.oceanRipple",
    "filter.gallery.basRelief",
    "filter.gallery.chalkCharcoal",
    "filter.gallery.charcoal",
    "filter.gallery.chrome",
    "filter.gallery.conteCrayon",
    "filter.gallery.graphicPen",
    "filter.gallery.halftonePattern",
    "filter.gallery.notePaper",
    "filter.gallery.photocopy",
    "filter.gallery.plaster",
    "filter.gallery.reticulation",
    "filter.gallery.stamp",
    "filter.gallery.tornEdges",
    "filter.gallery.waterPaper",
    "filter.gallery.glowingEdges",
    "filter.gallery.craquelure",
    "filter.gallery.grain",
    "filter.gallery.mosaicTiles",
    "filter.gallery.patchwork",
    "filter.gallery.stainedGlass",
    "filter.gallery.texturizer",
    // gradient.*
    "gradient.fill.create",
    "gradient.fill.set",
    "gradient.fill.stop",
    "gradient.fill.get",
    // layer.*
    "layer.fill.solidColor.set",
    "layer.fill.pattern.set",
    // type.*
    "type.create",
    "type.edit",
    "type.setStyle",
    "type.rasterize",
    "type.info",
    "type.fonts",
    "type.increaseSize",
    "type.decreaseSize",
    "type.increaseSizeMore",
    "type.decreaseSizeMore",
    "type.createSelection",
    // edit.*
    "edit.transform",
    // select.*
    "select.float",
    "select.drop",
    // shape.*
    "shape.create",
    "shape.edit",
    "shape.info",
    "shape.rasterize",
    // path.*
    "path.list",
    "path.info",
    "path.set",
    "path.delete",
    "path.transform",
    "path.clippingPath.set",
    "path.clippingPath.clear",
    "path.style.copyFill",
    "path.style.copyStroke",
    "path.style.pasteFill",
    "path.style.pasteStroke",
    "path.rename",
    "path.toSelection",
    // select.*
    "select.toWorkPath",
    // path.*
    "path.fill",
    "path.stroke",
    // layer.*
    "layer.vectorMask.add",
    "layer.vectorMask.fromPath",
    "layer.vectorMask.edit",
    "layer.vectorMask.delete",
    "layer.rasterize.vectorMask",
    "layer.vectorMask.revealAll",
    "layer.vectorMask.hideAll",
    "layer.vectorMask.currentPath",
    "layer.vectorMask.enabled",
    "layer.vectorMask.linked",
    "layer.rasterize.shape",
    "layer.combineShapes.unite",
    "layer.combineShapes.subtractFrontShape",
    "layer.combineShapes.intersectShapeAreas",
    "layer.combineShapes.excludeOverlappingShapes",
    "layer.combineShapes.mergeShapeComponents",
    // select.*
    "select.convertToShape",
    // layer.*
    "layer.vectorMask.info",
    // path.*
    "path.moveAnchors",
    "path.moveHandle",
    "path.bendSegment",
    "path.convertPoint",
    "path.addAnchor",
    "path.deleteAnchor",
    // select.*
    "select.quick",
    "select.object",
    "select.subject",
    "select.refineEdge",
    "select.focusArea",
    // layer.*
    "layer.removeBackground",
    // paint.*
    "paint.setSymmetry",
    "paint.symmetryFromPath",
    "paint.symmetryDisable",
    // layer.*
    "layer.new.layerViaCopy",
    "layer.new.layerViaCut",
    "layer.mergeVisible",
    // image.*
    "image.autoTone",
    "image.autoContrast",
    "image.autoColor",
    // view.*
    "view.newGuide",
    "view.moveGuide",
    "view.editGuide",
    "view.deleteGuide",
    "view.clearGuides",
    "view.setRulerOrigin",
    "view.resetRulerOrigin",
    // edit.*
    "edit.assignProfile",
    "edit.convertToProfile",
    "edit.profileInfo",
    // view.*
    "view.proofSetup",
    "view.proofColors",
    "view.gamutWarning",
    // paint.*
    "paint.pencil",
    "paint.mixerBrush",
    // brush.*
    "brush.mixer.load",
    "brush.mixer.clean",
    // paint.*
    "paint.colorReplacement",
    // brush.*
    "brush.presets.list",
    "brush.presets.setCurrent",
    "brush.defineFromSelection",
    "brush.get",
    // tools.*
    "tools.setBrush",
    // paint.*
    "paint.magicEraser",
    "paint.backgroundEraser",
    // swatches.*
    "swatches.list",
    "swatches.use",
    // paint.*
    "paint.cloneStamp",
    "paint.patternStamp",
    "paint.healingBrush",
    "paint.spotHealing",
    "paint.patch",
    "paint.contentAwareMove",
    "paint.remove",
    "paint.dodge",
    "paint.burn",
    "paint.sponge",
    "paint.blur",
    "paint.sharpen",
    "paint.smudge",
    "paint.historyBrush",
    "paint.redEye",
    // image.*
    "image.imageSize",
    "image.canvasSize",
    "image.crop",
    "image.trim",
    "image.mode.rgb",
    "image.mode.grayscale",
    "image.mode.cmyk",
    "image.mode.lab",
    "image.mode.bits8",
    "image.mode.bits16",
    "image.mode.bits32",
    "image.perspectiveCrop",
    // select.*
    "select.magicWand",
    "select.colorRange",
    "select.modify.border",
    "select.modify.smooth",
    "select.modify.expand",
    "select.modify.contract",
    "select.modify.feather",
    "select.grow",
    "select.similar",
    "select.lasso",
    "select.magneticLasso",
    "select.sky",
    "select.isolateLayers",
    "select.transformSelection",
    // paint.*
    "paint.bucket",
    "paint.gradient",
    // edit.*
    "edit.stroke",
    "edit.transform.rotate180",
    "edit.transform.rotate90Cw",
    "edit.transform.rotate90Ccw",
    "edit.transform.flipHorizontal",
    "edit.transform.flipVertical",
    // select.*
    "select.reselect",
    // image.*
    "image.adjustments.equalize",
    "image.revealAll",
    // layer.*
    "layer.new.layerFromBackground",
    "layer.layerStyle.copyLayerStyle",
    "layer.layerStyle.pasteLayerStyle",
    "layer.layerStyle.hideAllEffects",
    "layer.layerStyle.showAllEffects",
    "layer.layerMask.enabled",
    "layer.layerMask.linked",
    "layer.rasterize.layer",
    "layer.rasterize.allLayers",
    "layer.rasterize.type",
    "layer.rasterize.fillContent",
    "layer.rasterize.smartObject",
    "layer.delete.hiddenLayers",
    "layer.ungroupLayers",
    "layer.hideLayers",
    "layer.showLayers",
    "layer.toggleLockTransparency",
    "layer.showOnly",
    // filter.*
    "filter.blur.average",
    // layer.*
    "layer.new.backgroundFromLayer",
    // view.*
    "view.newGuideLayout",
    "view.newGuidesFromShape",
    "view.clearCanvasGuides",
    // type.*
    "type.antiAlias.none",
    "type.antiAlias.sharp",
    "type.antiAlias.crisp",
    "type.antiAlias.strong",
    "type.antiAlias.smooth",
    "type.antiAlias.windowsLcd",
    "type.antiAlias.windows",
    "type.orientation.horizontal",
    "type.orientation.vertical",
    "type.openType.standardLigatures",
    "type.openType.contextualAlternates",
    "type.openType.discretionaryLigatures",
    "type.openType.swash",
    "type.openType.oldstyle",
    "type.openType.stylisticAlternates",
    "type.openType.titlingAlternates",
    "type.openType.ornaments",
    "type.openType.ordinals",
    "type.openType.fractions",
    "type.createWorkPath",
    "type.convertToShape",
    "type.rasterizeTypeLayer",
    "type.convertToParagraphText",
    "type.convertToPointText",
    "type.warpText",
    "type.attachToPath",
    "type.detachFromPath",
    "type.updateAllTextLayers",
    "type.replaceAllMissingFonts",
    "type.resolveMissingFonts",
    "type.pasteLoremIpsum",
    "type.loadDefaultTypeStyles",
    "type.fillPath",
    "type.characterStyle.new",
    "type.characterStyle.duplicate",
    "type.characterStyle.delete",
    "type.characterStyle.rename",
    "type.characterStyle.set",
    "type.characterStyle.apply",
    "type.characterStyle.redefine",
    "type.characterStyle.clearOverride",
    "type.characterStyle.list",
    "type.paragraphStyle.new",
    "type.paragraphStyle.duplicate",
    "type.paragraphStyle.delete",
    "type.paragraphStyle.rename",
    "type.paragraphStyle.set",
    "type.paragraphStyle.apply",
    "type.paragraphStyle.redefine",
    "type.paragraphStyle.clearOverride",
    "type.paragraphStyle.list",
    // edit.*
    "edit.checkSpelling",
    // type.*
    "type.insertText",
    "type.hitTest",
    "type.caret",
    "type.navigate",
    // layer.*
    "layer.smartObjects.convertToSmartObject",
    // filter.*
    "filter.convertForSmartFilters",
    // layer.*
    "layer.smartObjects.newSmartObjectViaCopy",
    "layer.smartObjects.rasterize",
    "layer.smartObjects.updateModifiedContent",
    "layer.smartObjects.convertToLayers",
    "layer.smartObjects.convertToEmbedded",
    "layer.smartFilter.disableSmartFilters",
    "layer.smartFilter.deleteFilterMask",
    "layer.smartFilter.disableFilterMask",
    "layer.smartFilter.blendingOptions",
    "layer.smartFilter.clearSmartFilters",
    "layer.smartFilter.setVisible",
    "layer.smartFilter.setParams",
    "layer.smartFilter.delete",
    "layer.smartFilter.move",
    "layer.smartFilter.transfer",
    // select.*
    "select.allLayers",
    "select.deselectLayers",
    "select.findLayers",
    // layer.*
    "layer.selectLinkedLayers",
    "layer.align.topEdges",
    "layer.align.verticalCenters",
    "layer.align.bottomEdges",
    "layer.align.leftEdges",
    "layer.align.horizontalCenters",
    "layer.align.rightEdges",
    "layer.distribute.topEdges",
    "layer.distribute.verticalCenters",
    "layer.distribute.bottomEdges",
    "layer.distribute.leftEdges",
    "layer.distribute.horizontalCenters",
    "layer.distribute.rightEdges",
    "layer.distribute.horizontally",
    "layer.distribute.vertically",
    "layer.linkLayers",
    "layer.mergeLayers",
    "layer.new.groupFromLayers",
    "layer.arrange.reverse",
    "layer.lockLayers",
    "layer.renameLayer",
    // edit.*
    "edit.fade",
    "edit.contentAwareFill",
    "edit.deleteAndFillSelection",
    "edit.contentAwareScale",
    "edit.findAndReplaceText",
    "edit.fillForeground",
    "edit.fillBackground",
    "edit.fillForegroundPreserve",
    "edit.fillBackgroundPreserve",
    "edit.fillHistory",
    // document.*
    "document.sampleColor",
    // image.*
    "image.detectFaces",
    // view.*
    "view.colorSamplers.add",
    "view.colorSamplers.move",
    "view.colorSamplers.delete",
    "view.colorSamplers.clear",
    "view.colorSamplers.list",
    // tools.*
    "tools.decreaseBrushSize",
    "tools.increaseBrushSize",
    "tools.decreaseBrushHardness",
    "tools.increaseBrushHardness",
    "tools.previousBrush",
    "tools.nextBrush",
    "tools.firstBrush",
    "tools.lastBrush",
    "tools.rotateBrush",
    // layer.*
    "layer.stampVisible",
    "layer.stampDown",
    // edit.*
    "edit.autoAlignLayers",
    "edit.autoBlendLayers",
    // filter.*
    "filter.lensCorrection",
    "filter.adaptiveWideAngle",
    "filter.cameraRaw",
    "filter.vanishingPoint",
    // select.*
    "select.saveSelection",
    "select.loadSelection",
    "select.editInQuickMaskMode",
    // image.*
    "image.applyImage",
    "image.calculations",
    // channel.*
    "channel.new",
    "channel.newSpot",
    "channel.duplicate",
    "channel.delete",
    "channel.rename",
    "channel.move",
    "channel.target",
    "channel.setVisible",
    "channel.options",
    "channel.mergeSpot",
    "channel.split",
    "channel.merge",
    "channel.list",
    "channel.target.composite",
    "channel.target.layerMask",
    // select.*
    "select.loadLuminosity",
    // channel.*
    "channel.target.slot3",
    "channel.target.slot4",
    "channel.target.slot5",
    "channel.target.slot6",
    "channel.target.slot7",
    "channel.target.slot8",
    "channel.target.slot9",
    // image.*
    "image.adjustments.shadowsHighlights",
    "image.adjustments.replaceColor",
    "image.adjustments.matchColor",
    "image.adjustments.hdrToning",
    "image.adjustments.colorLookup.list",
    // layer.*
    "layer.layerMask.apply",
    "layer.layerMask.fromTransparency",
    "layer.layerMask.hideSelection",
    "layer.maskAllObjects",
    "layer.matting.defringe",
    "layer.matting.removeBlackMatte",
    "layer.matting.removeWhiteMatte",
    "layer.matting.colorDecontaminate",
    "layer.smartObjects.stackMode.entropy",
    "layer.smartObjects.stackMode.kurtosis",
    "layer.smartObjects.stackMode.maximum",
    "layer.smartObjects.stackMode.mean",
    "layer.smartObjects.stackMode.median",
    "layer.smartObjects.stackMode.minimum",
    "layer.smartObjects.stackMode.range",
    "layer.smartObjects.stackMode.skewness",
    "layer.smartObjects.stackMode.standardDeviation",
    "layer.smartObjects.stackMode.summation",
    "layer.smartObjects.stackMode.variance",
    "layer.smartObjects.stackMode.none",
    "layer.layerStyle.blendingOptions",
    "layer.layerStyle.globalLight",
    "layer.layerStyle.createLayer",
    "layer.layerStyle.scaleEffects",
    "layer.layerStyle.transferEffects",
    "layer.layerContentOptions",
    "layer.setLabelColor",
    "layer.xmpMetadata",
    // image.*
    "image.rotation.arbitrary",
    "image.mode.indexedColor",
    "image.mode.colorTable",
    "image.mode.bitmap",
    "image.mode.duotone",
    "image.mode.multichannel",
    // pattern.*
    "pattern.list",
    // layer.*
    "layer.newFillLayer.pattern",
    // brush.*
    "brush.texturePattern",
    // edit.*
    "edit.transform.warp",
    // layer.*
    "layer.smartObjects.warp",
    // layerComp.*
    "layerComp.new",
    "layerComp.update",
    "layerComp.apply",
    "layerComp.previous",
    "layerComp.next",
    "layerComp.delete",
    "layerComp.reorder",
    "layerComp.duplicate",
    "layerComp.rename",
    "layerComp.setComment",
    "layerComp.setOptions",
    "layerComp.restoreLastDocumentState",
    "layerComp.updateWarnings",
    "layerComp.list",
    // layer.*
    "layer.new.artboard",
    "layer.new.artboardFromGroup",
    "layer.new.artboardFromLayers",
    "layer.artboard.set",
    // view.*
    "view.clearSelectedArtboardGuides",
    // layer.*
    "layer.artboard.addBeside",
    // filter.*
    "filter.liquify",
    // edit.*
    "edit.puppetWarp",
    // layer.*
    "layer.smartObjects.puppetWarp",
    // edit.*
    "edit.perspectiveWarp",
    // layer.*
    "layer.smartObjects.perspectiveWarp",
    // image.*
    "image.analysis.setMeasurementScale",
    "image.analysis.selectDataPoints",
    "image.analysis.recordMeasurements",
    "image.analysis.rulerTool",
    "image.analysis.countTool",
    "image.analysis.placeScaleMarker",
    "image.analysis.straightenLayer",
    "image.analysis.info",
    // count.*
    "count.add",
    "count.remove",
    "count.move",
    "count.clear",
    "count.newGroup",
    "count.deleteGroup",
    "count.setGroup",
    // measurementLog.*
    "measurementLog.list",
    "measurementLog.delete",
    // notes.*
    "notes.add",
    "notes.set",
    "notes.delete",
    "notes.list",
    // view.*
    "view.proofSetup.workingCyanPlate",
    "view.proofSetup.workingMagentaPlate",
    "view.proofSetup.workingYellowPlate",
    "view.proofSetup.workingBlackPlate",
    "view.proofSetup.workingCmyPlate",
    "view.proofSetup.legacyMacintoshRgb",
    "view.proofSetup.colorBlindnessProtanopia",
    "view.proofSetup.colorBlindnessDeuteranopia",
    "view.thirtyTwoBitPreviewOptions",
    // gradient.*
    "gradient.presets.list",
    "gradient.presets.select",
    "gradient.presets.stop",
    "gradient.presets.apply",
    // pattern.*
    "pattern.presets.list",
    "pattern.presets.select",
    "pattern.presets.apply",
    // style.*
    "style.presets.list",
    "style.presets.apply",
    // shape.*
    "shape.presets.list",
    "shape.presets.place",
    "shape.presets.new",
    "shape.presets.edit",
    "shape.presets.reset",
    // tool.*
    "tool.presets.list",
    "tool.presets.select",
    // cloneSource.*
    "cloneSource.list",
    "cloneSource.select",
    "cloneSource.set",
    "cloneSource.resetTransform",
    "cloneSource.overlay",
    // filter.*
    "filter.render.pictureFrame",
    "filter.render.tree",
    // slice.*
    "slice.new",
    "slice.fromGuides",
    "slice.set",
    "slice.promote",
    "slice.delete",
    "slice.divide",
    "slice.list",
    // layer.*
    "layer.newLayerBasedSlice",
    // view.*
    "view.lockSlices",
    "view.clearSlices",
    // edit.*
    "edit.installedProfiles",
    // layer.*
    "layer.selectInRect",
    "layer.pickAt",
    "layer.selectAbove",
    "layer.selectBelow",
    "layer.selectTop",
    "layer.selectBottom",
    "layer.addAboveToSelection",
    "layer.addBelowToSelection",
    "layer.new.frame",
    "layer.new.frameFromLayers",
    // image.*
    "image.trap",
    // timeline.*
    "timeline.create",
    "timeline.delete",
    "timeline.setFrame",
    "timeline.nextFrame",
    "timeline.previousFrame",
    "timeline.setProps",
    "timeline.info",
    // layer.*
    "layer.videoLayers.newBlankVideoLayer",
    "layer.videoLayers.insertBlankFrame",
    "layer.videoLayers.duplicateFrame",
    "layer.videoLayers.deleteFrame",
    "layer.videoLayers.restoreFrame",
    "layer.videoLayers.restoreAllFrames",
    "layer.videoLayers.showAlteredVideo",
    "layer.videoLayers.rasterize",
    "layer.rasterize.video",
    "layer.videoLayers.replaceFootage",
    "layer.videoLayers.interpretFootage",
    "layer.videoLayers.reloadFrame",
    "layer.rasterize.layerStyle",
    // image.*
    "image.variables.define",
    "image.variables.dataSets",
    "image.applyDataSet",
    // variables.*
    "variables.list",
    // layer.*
    "layer.setExpanded",
    "layer.setEffectsExpanded",
    "layer.setEffectsVisible",
    // view.*
    "view.layerMask",
    // layer.*
    "layer.toggleMaskOverlay",
    "layer.layerMask.edit",
    // tool.*
    "tool.reset",
    // window.*
    "window.panel.group",
    "window.panel.layout",
    "window.panel.move",
    "window.panel.float",
    "window.panel.dock",
    "window.panel.activate",
    "window.panel.close",
    // view.*
    "view.rulers",
    "view.show.grid",
    "view.show.guides",
    "view.snap",
    "view.lockGuides",
    "view.extras",
    "view.show.targetPath",
    "view.zoomIn",
    "view.zoomOut",
    "view.fitOnScreen",
    "view.actualPixels",
    "view.rotateView",
    "view.resetView",
    // window.*
    "window.toggle.layers",
    "window.toggle.history",
    "window.toggle.properties",
    "window.toggle.color",
    "window.toggle.brushSettings",
    "window.toggle.brushes",
    "window.toggle.navigator",
    "window.toggle.toolbar",
    "window.togglePanels",
    "window.toggle.dock",
    "window.toggle.options",
    // tools.*
    "tools.select.move",
    "tools.select.artboard",
    "tools.select.rectMarquee",
    "tools.select.ellipseMarquee",
    "tools.select.singleRowMarquee",
    "tools.select.singleColumnMarquee",
    "tools.select.lasso",
    "tools.select.polygonLasso",
    "tools.select.magneticLasso",
    "tools.select.magicWand",
    "tools.select.crop",
    "tools.select.perspectiveCrop",
    "tools.select.eyedropper",
    "tools.select.colorSampler",
    "tools.select.ruler",
    "tools.select.note",
    "tools.select.count",
    "tools.select.brush",
    "tools.select.pencil",
    "tools.select.mixerBrush",
    "tools.select.colorReplacement",
    "tools.select.eraser",
    "tools.select.backgroundEraser",
    "tools.select.magicEraser",
    "tools.select.gradient",
    "tools.select.paintBucket",
    "tools.select.type",
    "tools.select.verticalType",
    "tools.select.horizontalTypeMask",
    "tools.select.verticalTypeMask",
    "tools.select.hand",
    "tools.select.rotateView",
    "tools.select.zoom",
    "tools.select.remove",
    "tools.select.spotHealing",
    "tools.select.healing",
    "tools.select.patch",
    "tools.select.contentAwareMove",
    "tools.select.redEye",
    "tools.select.cloneStamp",
    "tools.select.patternStamp",
    "tools.select.historyBrush",
    "tools.select.blur",
    "tools.select.sharpen",
    "tools.select.smudge",
    "tools.select.dodge",
    "tools.select.burn",
    "tools.select.sponge",
    "tools.select.quickSelection",
    "tools.select.objectSelection",
    "tools.select.pen",
    "tools.select.freeformPen",
    "tools.select.curvaturePen",
    "tools.select.addAnchorPoint",
    "tools.select.deleteAnchorPoint",
    "tools.select.convertPoint",
    "tools.select.pathSelection",
    "tools.select.directSelection",
    "tools.select.rectangle",
    "tools.select.ellipseShape",
    "tools.select.triangle",
    "tools.select.polygon",
    "tools.select.line",
    "tools.select.customShape",
    "tools.select.slice",
    "tools.select.sliceSelect",
    "tools.select.frame",
];

/// Commands the window itself provides (they take arguments, which the registry's commands do not, or answer in a shape of their own):
/// `inspect` reads the document (name, size, mode, depth), its layers as a tree (id, name, kind, visible, opacity, blend, bounds, selected) to a
/// depth, the selection and what Undo would say; `render` reads the composite (or a layer, by id) as a PNG, base64, at most 1600 pixels on a side.
pub const ARGUMENT_COMMANDS: &[&str] = &["inspect", "render"];

/// Every command the app has and the assistant may NOT run, with the reason: together with `ASSISTANT_COMMANDS` this is the whole command set of the window
/// (a test holds the two to the registry, so a command can neither be forgotten nor listed twice).
pub const LEFT_OUT: &[(&str, &[&str])] = &[
    (
        "Files move only through the host: open, save, export, place, print, close and the like are the host's, never a command id the assistant runs.",
        &[
            "file.new", "file.close", "layer.newAdjustmentLayer.colorLookup", "image.adjustments.colorLookup", "layer.layerStyle.contours.load",
            "layer.layerStyle.contours.save", "filter.distort.displace", "file.newFromClipboard", "brush.presets.importAbr",
            "brush.presets.importImage", "file.scripts.deleteAllEmptyLayers", "file.closeAll", "file.closeOthers", "file.revealInFinder",
            "file.revert", "file.saveACopy", "file.openAs", "file.placeEmbedded", "file.placeLinked", "file.fileInfo", "file.automate.fitImage",
            "file.automate.conditionalModeChange", "file.automate.batch", "file.scripts.imageProcessor", "file.scripts.loadFilesIntoStack",
            "file.scripts.flattenAllLayerEffects", "file.scripts.flattenAllMasks", "file.export.layersToFiles", "file.export.colorLookupTables",
            "file.exrParts", "file.openExrParts", "layer.smartObjects.saveContents", "layer.smartObjects.replaceContents",
            "layer.smartObjects.exportContents", "layer.smartObjects.relinkToFile", "layer.smartObjects.updateChanged",
            "layer.smartObjects.updateAllModifiedContent", "layer.smartObjects.convertToLinked", "edit.presets.exportImportPresets",
            "file.automate.photomerge", "file.automate.mergeToHdrPro", "file.automate.cropAndStraightenPhotos", "file.automate.lensCorrection",
            "layer.smartObjects.revealInFinder", "layer.quickExportAsPng", "layer.exportAs", "file.export.layerCompsToFiles",
            "file.export.artboardsToFiles", "file.export.artboardsToPdf", "file.import.notes", "file.export.presets.list", "file.export.presets.get",
            "file.export.presets.save", "file.export.presets.delete", "filter.render.flame", "file.export.saveForWebLegacy",
            "file.export.exportPreferences", "file.export.quickExport", "file.generate.imageAssets", "file.automate.contactSheetII",
            "file.automate.createDroplet", "file.automate.runDroplet", "file.scripts.statistics", "file.scripts.browse",
            "file.scripts.scriptEventsManager", "file.print", "file.print.printers", "file.print.page", "file.printers", "file.printOneCopy",
            "file.package", "file.export.pathsToIllustrator", "edit.presets.migratePresets", "file.import.videoFramesToLayers",
            "file.export.renderVideo", "file.import.wiaSupport", "file.import.variableDataSets", "file.export.dataSetsAsFiles", "file.open",
            "file.save", "file.saveAs", "file.exit", "file.export.exportAs", "file.export.quickExportAsPng",
        ],
    ),
    (
        "Undo, redo and the history belong to the person: a run is one undo step, and undoing it is theirs.",
        &[
            "edit.undo", "edit.redo", "edit.toggleLastState", "edit.purge.undo", "edit.purge.clipboard", "edit.purge.histories",
            "edit.purge.videoCache", "edit.purge.all", "history.undo", "history.redo", "history.deleteState", "history.newDocument",
            "history.newSnapshot", "history.selectSnapshot", "history.renameSnapshot", "history.deleteSnapshot", "history.stepForward",
            "history.stepBackward", "history.clear", "history.setSource", "history.selectState",
        ],
    ),
    (
        "The window shows the host's one file: commands that open, switch or duplicate another document are refused.",
        &[
            "document.activate", "document.move", "image.duplicate", "layer.smartObjects.editContents", "layer.copyToDocument",
            "window.newWindowForDocument", "window.nextDocument", "window.previousDocument",
        ],
    ),
    (
        "The system clipboard is the person's (copy, cut and paste read or write what they hold outside the app).",
        &[
            "edit.cut", "edit.copy", "edit.copyMerged", "edit.paste", "edit.pasteSpecial.pasteInPlace", "edit.pasteSpecial.pasteInto",
            "edit.pasteSpecial.pasteOutside",
        ],
    ),
    (
        "Interactive: it begins a pointer gesture, a dialog or a text-editing session that only a person's pointer can finish (use the command that takes its numbers: edit.transform, select.rect, type.create ...).",
        &[
            "edit.transform.again", "edit.transform.againCopy", "edit.transform.splitWarpCrosswise", "edit.transform.splitWarpHorizontally",
            "edit.transform.splitWarpVertically", "edit.transform.removeWarpSplit", "edit.transform.warpGrid", "ui.symmetryTransform",
            "tool.resetAll", "edit.freeTransform", "edit.transform.scale", "edit.transform.rotate", "edit.transform.skew", "edit.transform.distort",
            "edit.transform.perspective", "select.selectAndMask", "edit.freeTransformCopy", "type.editText", "edit.search",
        ],
    ),
    (
        "Preferences, appearance and changes to the person's preset libraries (brushes, swatches, gradients, styles, patterns, tool and document presets: stored in their browser) are theirs, not the image's. Picking from a library is allowed.",
        &[
            "edit.colorSettings", "color.profileMismatch", "brush.presets.save", "brush.presets.update", "brush.presets.delete",
            "brush.presets.rename", "brush.presets.move", "brush.presets.moveGroup", "brush.presets.renameGroup", "brush.presets.deleteGroup",
            "gradient.presets.importGrd", "style.presets.importAsl", "style.presets.exportAsl", "swatches.add", "swatches.delete", "swatches.rename",
            "swatches.move", "swatches.newGroup", "swatches.renameGroup", "swatches.deleteGroup", "swatches.reset", "swatches.import",
            "swatches.export", "prefs.get", "prefs.set", "prefs.reset", "edit.preferences.general", "edit.preferences.interface",
            "edit.preferences.workspace", "edit.preferences.tools", "edit.preferences.historyLog", "edit.preferences.fileHandling",
            "edit.preferences.export", "edit.preferences.performance", "edit.preferences.scratchDisks", "edit.preferences.cursors",
            "edit.preferences.transparencyAndGamut", "edit.preferences.unitsAndRulers", "edit.preferences.guidesGridAndSlices",
            "edit.preferences.plugIns", "edit.preferences.type", "edit.preferences.enhancedControls", "edit.preferences.rawDefaults",
            "edit.preferences.integrations", "edit.keyboardShortcuts", "edit.menus", "edit.toolbar", "edit.defineBrushPreset",
            "edit.defineCustomShape", "edit.presets.presetManager", "edit.definePattern", "pattern.rename", "pattern.delete", "pattern.import",
            "pattern.export", "gradient.presets.new", "gradient.presets.edit", "gradient.presets.reset", "pattern.presets.new",
            "pattern.presets.edit", "style.presets.new", "style.presets.edit", "style.presets.reset", "tool.presets.new", "tool.presets.edit",
            "tool.presets.reset", "document.presets.list", "document.presets.get", "document.presets.save", "document.presets.delete",
            "document.recent.list", "document.recent.add", "view.screenMode.cycle", "view.darkenInterface", "view.lightenInterface",
            "window.theme.toggle", "window.theme.pro", "window.theme.proMedium", "window.theme.studio", "window.theme.solarizedDark",
            "window.theme.adwaitaDark", "window.theme.breezeDark", "window.theme.softDark", "window.theme.proLightGray", "window.theme.proLight",
            "window.theme.studioLight", "window.theme.classic", "window.theme.adwaita", "window.theme.breezeLight", "window.theme.softLight",
            "window.theme.system",
        ],
    ),
    (
        "Recorded actions, plug-ins, updates and background jobs: scripting and app plumbing, not the image (an action can replay a file command).",
        &[
            "actions.load", "actions.save", "actions.stopMessage", "actions.photoshopStep", "actions.insert", "actions.dialog",
            "actions.conditional", "actions.assignSet", "updates.status", "jobs.list", "jobs.cancel", "plugin.list", "plugin.install",
            "plugin.remove", "plugin.reload", "plugin.run", "actions.move", "actions.list", "actions.get", "actions.record", "actions.stop",
            "actions.play", "actions.delete", "actions.toggle", "actions.rename", "actions.setColor",
        ],
    ),
    (
        "Its name is a file operation (open, save, export, import, print, library, plug-in, script).",
        &[
            "type.saveDefaultTypeStyles", "measurementLog.export", "layer.videoLayers.newVideoLayerFromFile",
        ],
    ),
    (
        "Opens a web page or an information dialog (help), or answers one of the app's own prompts.",
        &[
            "help.checkForUpdates", "help.discord", "help.website", "help.artcraftWebsite", "help.github", "help.reportIssue", "help.systemInfo",
            "help.about",
        ],
    ),
];

/// Every command id a run may name, in the order the host's registry declares them.
pub fn allowed() -> Vec<&'static str> {
    ASSISTANT_COMMANDS.iter().chain(ARGUMENT_COMMANDS).copied().collect()
}

/// The most steps one run may hold.
pub const MAX_STEPS: usize = 50;
/// The longest label of a run.
pub const MAX_LABEL: usize = 120;
/// The most levels `inspect` walks into.
const MAX_INSPECT_DEPTH: usize = 8;
/// The longest side of a `render`.
const MAX_RENDER_SIDE: f64 = 1600.0;
/// Argument names that would point a command at a file, a colour profile on disk or a network address: refused anywhere in a step's arguments.
const FORBIDDEN_ARGS: &[&str] = &["path", "paths", "dir", "folder", "file", "url", "uri", "href", "server", "data", "profile"];

/// The first forbidden argument name anywhere in `v`: a file or address, or a private (`__`) key the interface uses between its own dialogs.
fn forbidden_arg(v: &Value) -> Option<String> {
    match v {
        Value::Object(o) => {
            o.iter().find_map(|(k, x)| if FORBIDDEN_ARGS.contains(&k.as_str()) || k.starts_with("__") { Some(k.clone()) } else { forbidden_arg(x) })
        }
        Value::Array(a) => a.iter().find_map(forbidden_arg),
        _ => None,
    }
}

/// A run, judged: its label and its steps (the command id and its arguments).
#[derive(Debug, Clone, PartialEq)]
pub struct Run {
    pub label: String,
    pub steps: Vec<(String, Value)>,
}

/// Judge a `command.batch` request before anything runs.
pub fn judge(params: &Value) -> Result<Run, String> {
    let label = params.get("label").and_then(Value::as_str).map(str::trim).unwrap_or("");
    if label.chars().count() > MAX_LABEL {
        return Err(format!("label-too-long: a run's label is at most {MAX_LABEL} characters"));
    }
    let steps = params.get("steps").and_then(Value::as_array).ok_or("run-empty: a run states its steps")?;
    if steps.is_empty() {
        return Err("run-empty: a run states at least one command".into());
    }
    if steps.len() > MAX_STEPS {
        return Err(format!("run-too-long: a run is at most {MAX_STEPS} steps"));
    }
    let mut named = Vec::with_capacity(steps.len());
    for (i, step) in steps.iter().enumerate() {
        let name = step.get("name").and_then(Value::as_str).unwrap_or("");
        if !ASSISTANT_COMMANDS.contains(&name) && !ARGUMENT_COMMANDS.contains(&name) {
            return Err(format!("command-not-allowed: Photo does not allow the command '{name}' -- nothing was run"));
        }
        let args = step.get("args").cloned().unwrap_or_else(|| json!({}));
        if !(args.is_object() || args.is_null()) {
            return Err(format!("argument-not-allowed: step {} ('{name}') takes its arguments as an object -- nothing was run", i + 1));
        }
        if let Some(k) = forbidden_arg(&args) {
            return Err(format!("argument-not-allowed: step '{name}' names '{k}', and a run does not reach files or addresses -- nothing was run"));
        }
        named.push((name.to_string(), args));
    }
    let label = if label.is_empty() { "Assistant" } else { label };
    Ok(Run { label: label.to_string(), steps: named })
}

/// Replace every `{"$step": n}` / `{"$step": n, "field": "..."}` in `v` with that field (default `id`, else `layer`) of
/// `results[n]`, the result of an earlier step.
pub fn resolve_step_refs(v: &Value, results: &[Value], step: usize, depth: usize) -> Result<Value, String> {
    if depth > 64 {
        return Err(format!("step {}: arguments nest deeper than 64 levels", step + 1));
    }
    match v {
        Value::Object(o) if o.contains_key("$step") => {
            if let Some(k) = o.keys().find(|k| *k != "$step" && *k != "field") {
                return Err(format!("step {}: a `$step` reference takes only `field`, not `{k}`", step + 1));
            }
            let n = o.get("$step").and_then(Value::as_u64).ok_or_else(|| format!("step {}: `$step` must be a step index (0, 1, ...)", step + 1))?;
            let Some(result) = usize::try_from(n).ok().filter(|n| *n < step).and_then(|n| results.get(n)) else {
                return Err(format!("step {}: `$step` {n} must be an earlier step", step + 1));
            };
            let field = match o.get("field") {
                None => None,
                Some(Value::String(f)) => Some(f.as_str()),
                Some(_) => return Err(format!("step {}: `field` must be a string", step + 1)),
            };
            let found = match field {
                Some(f) => result.get(f),
                None => result.get("id").or_else(|| result.get("layer")),
            };
            found.cloned().ok_or_else(|| format!("step {}: step {n}'s result has no `{}`", step + 1, field.unwrap_or("id")))
        }
        Value::Object(o) => {
            let mut out = serde_json::Map::with_capacity(o.len());
            for (k, x) in o {
                out.insert(k.clone(), resolve_step_refs(x, results, step, depth + 1)?);
            }
            Ok(Value::Object(out))
        }
        Value::Array(a) => a.iter().map(|x| resolve_step_refs(x, results, step, depth + 1)).collect::<Result<Vec<_>, _>>().map(Value::Array),
        _ => Ok(v.clone()),
    }
}

/// Run a judged run: every step through `step` (the registry, for the interface; the engine alone, for a headless session), the
/// whole of it as ONE history step named by the label, or none of it. `session` reaches the engine session inside `c`.
pub fn run_batch<C>(
    c: &mut C,
    session: impl Fn(&mut C) -> &mut Session,
    mut step: impl FnMut(&mut C, &str, &Value) -> Result<Value, String>,
    params: &Value,
) -> Result<Value, String> {
    let run = judge(params)?;
    if session(c).active().is_none() {
        return Err("no-document: no image is open in this window".into());
    }
    let depth = session(c).active().map_or(0, |d| d.history.past_len());
    let mark = session(c).group_begin().map_err(|e| e.to_string())?;
    let mut results: Vec<Value> = Vec::with_capacity(run.steps.len());
    let mut failed: Option<String> = None;
    for (i, (name, args)) in run.steps.iter().enumerate() {
        let outcome = resolve_step_refs(args, &results, i, 0).and_then(|args| step(c, name, &args));
        match outcome {
            Ok(result) => results.push(result),
            Err(e) => {
                failed = Some(format!("step-failed: step {} ({name}): {e} -- nothing was changed", i + 1));
                break;
            }
        }
    }
    let keep = failed.is_none();
    session(c).group_end(mark, &run.label, keep).map_err(|e| e.to_string())?;
    if let Some(e) = failed {
        return Err(e);
    }
    let steps: Vec<Value> = run.steps.iter().zip(results).map(|((name, _), result)| json!({ "name": name, "result": result })).collect();
    let undo_step = session(c).active().is_some_and(|d| d.history.past_len() > depth);
    Ok(json!({ "steps": steps, "undoStep": undo_step }))
}

impl Session {
    /// One step of a run: the window's own commands here, every other id through the registry.
    pub fn run_host_step(&mut self, name: &str, args: &Value) -> Result<Value, String> {
        match name {
            "inspect" | "render" => self.run_argument_step(name, args),
            other => self.execute(other, args.clone()).map_err(|e| e.to_string()),
        }
    }

    /// The window's own commands (`ARGUMENT_COMMANDS`).
    pub fn run_argument_step(&mut self, name: &str, args: &Value) -> Result<Value, String> {
        match name {
            "inspect" => self.inspect_image(args),
            "render" => self.render_image(args),
            other => Err(format!("unknown window command '{other}'")),
        }
    }

    /// One `command.batch` on this session alone (a headless session, the node build the assistant tests drive): all of it as ONE
    /// history step, or none of it.
    pub fn run_command_batch(&mut self, params: &Value) -> Result<Value, String> {
        run_batch(self, |s| s, |s, name, args| s.run_host_step(name, args), params)
    }

    /// The active document encoded as the file `name` names (its extension), the way a Save would write it: the bytes the host
    /// stores as the next version.
    pub fn export_active(&self, name: &str) -> Result<Vec<u8>, String> {
        let st = self.active().ok_or("no-document: no image is open")?;
        crate::file_cmds::encode(&st.doc, name, crate::file_cmds::SaveOpts::default()).map(|(bytes, _)| bytes).map_err(|e| e.to_string())
    }

    fn inspect_image(&mut self, args: &Value) -> Result<Value, String> {
        let st = self.active().ok_or("no-document: no image is open in this window")?;
        let depth = args.get("depth").and_then(Value::as_u64).map_or(4, |d| (d as usize).min(MAX_INSPECT_DEPTH));
        let full = crate::inspect::document(st);
        let mut layers = full["layers"].clone();
        prune_layers(&mut layers, depth);
        Ok(json!({
            "name": full["name"],
            "width": full["width"],
            "height": full["height"],
            "mode": full["mode"],
            "depth": full["depth"],
            "activeLayer": full["activeLayer"],
            "selectedLayers": full["selectedLayers"],
            "hasSelection": full["hasSelection"],
            "selectionBounds": full["selectionBounds"],
            "layers": layers,
            "canUndo": full["canUndo"],
            "undo": st.history.undo_label(),
        }))
    }

    fn render_image(&mut self, args: &Value) -> Result<Value, String> {
        let st = self.active().ok_or("no-document: no image is open in this window")?;
        let mut preview = Session::new();
        preview.add_document((*st.doc).clone(), None);
        if let Some(layer) = args.get("layer").and_then(Value::as_u64) {
            if st.doc.layer(photocraft_doc::LayerId(layer)).is_none() {
                return Err(format!("no layer {layer}"));
            }
            preview.execute("layer.showOnly", json!({ "layer": layer })).map_err(|e| e.to_string())?;
        }
        let (w, h) = (f64::from(st.doc.size.width), f64::from(st.doc.size.height));
        let side = w.max(h);
        if side > MAX_RENDER_SIDE {
            let percent = (MAX_RENDER_SIDE / side * 100.0).max(0.1);
            preview.execute("image.imageSize", json!({ "percent": percent })).map_err(|e| e.to_string())?;
        }
        let doc = &preview.active().ok_or("no-document")?.doc;
        let (bytes, _) = crate::file_cmds::encode(doc, "render.png", crate::file_cmds::SaveOpts::default()).map_err(|e| e.to_string())?;
        Ok(json!({
            "mime": "image/png",
            "width": doc.size.width,
            "height": doc.size.height,
            "bytes": bytes.len(),
            "png": photocraft_paint::tile::b64_encode(&bytes),
        }))
    }
}

/// Cut the layer tree off below `depth` levels (a group past the cut keeps its child count).
fn prune_layers(layers: &mut Value, depth: usize) {
    let Some(list) = layers.as_array_mut() else { return };
    for l in list {
        if let Some(children) = l.get_mut("children") {
            if depth <= 1 {
                let n = children.as_array().map_or(0, Vec::len);
                l["childCount"] = json!(n);
                if let Some(o) = l.as_object_mut() {
                    o.remove("children");
                }
            } else {
                prune_layers(children, depth - 1);
            }
        }
    }
}

#[cfg(test)]
mod tests;
