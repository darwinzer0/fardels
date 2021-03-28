import React, { Component } from 'react';
import { connect } from 'react-redux';
import * as C from './Const'

import { withStyles } from '@material-ui/core/styles';
import Button from '@material-ui/core/Button';
import Dialog from '@material-ui/core/Dialog';
import DialogActions from '@material-ui/core/DialogActions';
import DialogContent from '@material-ui/core/DialogContent';
import DialogContentText from '@material-ui/core/DialogContentText';
import DialogTitle from '@material-ui/core/DialogTitle';
import TextField from '@material-ui/core/TextField';
import Typography from '@material-ui/core/Typography';
import ThumbUpTwoToneIcon from '@material-ui/icons/ThumbUpTwoTone';
import ThumbDownTwoToneIcon from '@material-ui/icons/ThumbDownTwoTone';


import { executeComment,
         executeUpvote,
         executeDownvote,
         setFardelViewDialogOpen, } from './store/Actions';

const useStyles = theme => ({
  button: {
    margin: theme.spacing(1),
  },
  buttonBox: {
    width: '100%',
  },
  root: {
    display: 'flex',
    flexWrap: 'wrap',
  },
});

class FardelViewDialog extends Component {

  constructor(props) {
    super(props);
    this.state = { 
      comment: '',
    };
  }

  handleUpvote() {
    const { selectedFardel, viewedFardels, executeUpvote } = this.props;
    console.log("upvote");
    if (selectedFardel >= 0 && viewedFardels[selectedFardel]) {
      executeUpvote(viewedFardels[selectedFardel].id);
    }
  }

  handleDownvote() {
    const { selectedFardel, viewedFardels, executeDownvote } = this.props;
    console.log("downvote");
    if (selectedFardel >= 0 && viewedFardels[selectedFardel]) {
      executeDownvote(viewedFardels[selectedFardel].id);
    }
  }

  handleComment() {
    const { selectedFardel, viewedFardels } = this.props;
    console.log("comment");
    if (selectedFardel >= 0 && viewedFardels[selectedFardel] && this.state.comment) {
      executeComment(viewedFardels[selectedFardel].id, this.state.comment);
    }
    this.setState({
      comment: '',
    })
  }

  handleCommentInput(event) {
    this.setState({
      comment: event.target.value,
    });
  }

  handleClose() {
    const { setFardelViewDialogOpen } = this.props;

    setFardelViewDialogOpen(false);
  }

  render() {
    const { classes, fardelViewDialogOpen, selectedFardel, viewedFardels } = this.props;

    let fardel = null;
    if (selectedFardel >= 0) {
      fardel = viewedFardels[selectedFardel];
    }
    let showContents = fardel && (!fardel.packed);

    return (
      <Dialog
        fullWidth
        maxWidth={"sm"}
        open={fardelViewDialogOpen}
        onClose={() => this.handleClose()}
        aria-labelledby="fardel-view-dialog"
      >
        <DialogTitle id="fardel-view-dialog-title">View Fardel</DialogTitle>
        <DialogContent>
          <DialogContentText>
          { showContents &&
          <Typography>
            <strong>Contents</strong><br />
            {fardel.contents_text}<br />
            {fardel.ipfs_cid && 
              <div><a target="_blank" href={"https://cloudflare-ipfs.com/ipfs/"+fardel.ipfs_cid}>IPFS link</a><br /></div>
            }
            {fardel.passphrase &&
              <div><strong>Passphrase</strong><br />
                   {fardel.passphrase}<br /></div>
            }
          </Typography>
          }
          </DialogContentText>
          <TextField
              autoFocus
              margin="dense"
              id="comment"
              label="Comment"
              type="text"
              fullWidth
              value={this.state.comment}
              onChange={(e) => this.handleCommentInput(e)}
           />
        </DialogContent>
        <DialogActions>
          <Button 
            startIcon={<ThumbUpTwoToneIcon />}
            onClick={() => this.handleUpvote()}
           >
            Upvote
          </Button>
          <Button 
            startIcon={<ThumbDownTwoToneIcon />}
            onClick={() => this.handleDownvote()}
           >
            Upvote
          </Button>
          <Button onClick={() => this.handleComment()}>
            Comment
          </Button>
          <Button onClick={() => this.handleClose()} color="primary">
            Close
          </Button>
        </DialogActions>
      </Dialog>
    );
  }
}

const mapStateToProps = (state) => {
  return {
    viewedFardels: state.viewedFardels,
    fardelViewDialogOpen: state.fardelViewDialogOpen,
    selectedFardel: state.selectedFardel,
    viewedFardels: state.viewedFardels,
  }
};

const mapDispatchToProps = (dispatch) => {
  return ({
    executeComment: (fardelId, comment) => { dispatch(executeComment(fardelId, comment)) },
    executeUpvote: (fardelId) => { dispatch(executeUpvote(fardelId)) },
    executeDownvote: (fardelId) => { dispatch(executeDownvote(fardelId)) },
    setFardelViewDialogOpen: (fardelViewDialogOpen) =>
        { dispatch(setFardelViewDialogOpen(fardelViewDialogOpen)) },
  });
}

export default connect(mapStateToProps, mapDispatchToProps)(withStyles(useStyles)(FardelViewDialog));
